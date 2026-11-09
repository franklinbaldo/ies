"""RED #264: ensaio sintético de atribuição/restauração, sem rede."""
import json
import tempfile
import unittest
from pathlib import Path
from ensaio_processo_atribuir import ensaiar

NUP = "23106.001559/2026-11"
USUARIO = "aluno2"
UNIDADE = "TESTE"
ORIGEM = "https://sei.treinamento.unb.br/sei"

def previa(remover=False):
    return 0, dict(nup=NUP, id_procedimento="42", unidade=UNIDADE,
                   usuario_atual=USUARIO if remover else "",
                   usuario_pedido=None if remover else USUARIO,
                   confirmado=False, alterado=False, requisicoes=4)

def confirmado(remover=False):
    return 0, dict(nup=NUP, id_procedimento="42", unidade=UNIDADE,
                   usuario_pedido=None if remover else USUARIO,
                   confirmado=True, alterado=True, requisicoes=6)

def leitura(remover=False):
    return 0, dict(numero=NUP, id_procedimento="42", requisicoes=2,
                   aberto_em=[dict(sigla=UNIDADE, atribuido_a=None if remover
                                   else dict(sigla=USUARIO))])

def roteiro():
    return [previa(), confirmado(), leitura(), previa(True),
            confirmado(True), leitura(True)]

class EnsaioAtribuicao(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.registro = Path(self.temp.name) / "unico.json"
        self.chamadas = []
        self.pausas = []

    def rodar(self, dados=None):
        respostas = iter(roteiro() if dados is None else dados)
        def executar(_ies, argumentos):
            self.chamadas.append(argumentos)
            return next(respostas)
        return ensaiar(ies="ies-falso", registro=self.registro, nup=NUP,
                       usuario=USUARIO, origem=ORIGEM, executar=executar,
                       pausa=self.pausas.append)

    def test_ciclo_com_dois_posts_unicos_releituras_e_residuo(self):
        codigo, relatorio = self.rodar()
        self.assertEqual((codigo, relatorio["estado"]), (0, "PASS"))
        base = ["processo", "atribuir", NUP, "--usuario", USUARIO]
        remover = ["processo", "atribuir", NUP, "--remover"]
        ler = ["processo", NUP, "--sem-documentos", "--sem-andamentos",
               "--sem-marcadores", "--sem-propriedades"]
        self.assertEqual(self.chamadas, [base, base + ["--confirmar"], ler,
                         remover, remover + ["--confirmar"], ler])
        self.assertEqual(self.pausas, [30] * 5)
        self.assertEqual(self.registro.stat().st_mode & 0o777, 0o600)
        self.assertEqual(json.loads(self.registro.read_text())["requisicoes"],
                         [4, 6, 2, 4, 6, 2])
        self.assertEqual(relatorio["residuo"], "sem_atribuicao")

    def test_registro_existente_recusa_sem_nova_requisicao(self):
        self.rodar()
        self.chamadas.clear()
        codigo, relatorio = self.rodar()
        self.assertEqual((codigo, relatorio["estado"]), (2, "BLOQUEADO"))
        self.assertEqual(self.chamadas, [])

    def test_previa_sem_estado_e_atribuicao_original_recusadas(self):
        for estado in (None, "outro"):
            with self.subTest(estado=estado):
                resposta = previa()
                if estado is None:
                    resposta[1].pop("usuario_atual")
                else:
                    resposta[1]["usuario_atual"] = estado
                codigo, relatorio = self.rodar([resposta])
                self.assertEqual((codigo, relatorio["estado"]), (1, "PREVIA_FAIL"))
                self.assertEqual(len(self.chamadas), 1)
                self.chamadas.clear()
                self.assertFalse(self.registro.exists())

    def test_post_sem_resposta_nao_repete_escrita(self):
        dados = roteiro()
        dados[1] = 3, {"subcodigo": "efeito_incerto_na_atribuicao", "requisicoes": 6}
        codigo, relatorio = self.rodar(dados)
        self.assertEqual((codigo, relatorio["estado"]), (3, "EFEITO_INCERTO"))
        self.assertEqual(len(self.chamadas), 2)
        self.assertEqual(relatorio["requisicoes"], [4, 6])
        self.assertTrue(self.registro.exists())

    def test_post_inconclusivo_conserva_atribuicao_relida_sem_declarar_sucesso(self):
        dados = roteiro()
        dados[1] = 3, {"subcodigo": "efeito_incerto_na_atribuicao",
                       "requisicoes": 6, "atribuicao_relida": USUARIO}
        codigo, relatorio = self.rodar(dados)
        self.assertEqual((codigo, relatorio["estado"]), (3, "EFEITO_INCERTO"))
        self.assertEqual(relatorio["residuo"], "possivel_atribuicao")
        self.assertEqual(relatorio["atribuicao_relida"], USUARIO)
        self.assertEqual(json.loads(self.registro.read_text())["atribuicao_relida"], USUARIO)
        self.assertEqual(len(self.chamadas), 2)

    def test_remocao_inconclusiva_conserva_ausencia_observada_sem_sucesso(self):
        dados = roteiro()
        dados[4] = 3, {"subcodigo": "efeito_incerto_na_atribuicao",
                       "requisicoes": 6, "atribuicao_relida": ""}
        codigo, relatorio = self.rodar(dados)
        self.assertEqual((codigo, relatorio["estado"]), (3, "EFEITO_INCERTO"))
        self.assertEqual(relatorio["residuo"], "possivel_remocao")
        self.assertEqual(relatorio["atribuicao_relida"], "")
        self.assertEqual(json.loads(self.registro.read_text())["atribuicao_relida"], "")
        self.assertEqual(len(self.chamadas), 5)

    def test_releitura_nao_observada_permanece_nula_no_registro(self):
        dados = roteiro()
        dados[1] = 3, {"requisicoes": 6, "atribuicao_relida": None}
        codigo, relatorio = self.rodar(dados)
        self.assertEqual((codigo, relatorio["estado"]), (3, "EFEITO_INCERTO"))
        self.assertIn("atribuicao_relida", relatorio)
        self.assertIsNone(relatorio["atribuicao_relida"])
        self.assertNotEqual(relatorio["residuo"], "sem_atribuicao")
        self.assertEqual(len(self.chamadas), 2)

    def test_post_inconclusivo_nao_publica_valor_alheio_ou_dado_nao_tipado(self):
        for valor in ("https://example.invalid/segredo", ["aluno2"]):
            with self.subTest(valor=valor):
                dados = roteiro()
                dados[1] = 3, {"subcodigo": "efeito_incerto_na_atribuicao",
                               "requisicoes": 6, "atribuicao_relida": valor}
                codigo, relatorio = self.rodar(dados)
                self.assertEqual(codigo, 3)
                self.assertNotIn("atribuicao_relida", relatorio)
                self.assertNotIn("example.invalid", self.registro.read_text())
                self.registro.unlink()
                self.chamadas.clear()

    def test_releitura_de_outro_processo_nao_autoriza_remover(self):
        dados = roteiro()
        dados[2][1]["numero"] = "outro"
        codigo, relatorio = self.rodar(dados)
        self.assertEqual((codigo, relatorio["estado"]), (3, "VERIFICACAO_INCONCLUSIVA"))
        self.assertEqual(len(self.chamadas), 3)

    def test_previa_de_remocao_divergente_interrompe(self):
        dados = roteiro()
        dados[3][1]["usuario_atual"] = "concorrente"
        codigo, relatorio = self.rodar(dados)
        self.assertEqual((codigo, relatorio["estado"]), (3, "REMOCAO_BLOQUEADA"))
        self.assertEqual(len(self.chamadas), 4)

    def test_releitura_ausente_nao_eh_remocao(self):
        dados = roteiro()
        dados[5][1]["aberto_em"] = None
        codigo, relatorio = self.rodar(dados)
        self.assertEqual((codigo, relatorio["estado"]), (3, "REMOCAO_INCONCLUSIVA"))
        self.assertEqual(len(self.chamadas), 6)

    def test_causa_inesperada_e_contagem_invalida_nao_geram_pass(self):
        dados = roteiro()
        dados[4][1]["requisicoes"] = "https://example.invalid/segredo"
        codigo, relatorio = self.rodar(dados)
        self.assertEqual((codigo, relatorio["estado"]), (3, "EFEITO_INCERTO"))
        self.assertEqual(len(self.chamadas), 5)
        self.assertNotIn("example.invalid", self.registro.read_text())

    def test_timeout_no_post_preserva_trava_sem_retry(self):
        chamadas = []
        def executar(_ies, args):
            chamadas.append(args)
            if len(chamadas) == 1:
                return previa()
            raise OSError("timeout sintético")
        codigo, relatorio = ensaiar(
            ies="ies-falso", registro=self.registro, nup=NUP,
            usuario=USUARIO, origem=ORIGEM, executar=executar, pausa=lambda _: None)
        self.assertEqual((codigo, relatorio["estado"]), (3, "EFEITO_INCERTO"))
        self.assertEqual(len(chamadas), 2)
        self.assertTrue(self.registro.exists())


    def rodar_com_falha_fsync(self, numero_da_chamada):
        import os
        from unittest.mock import patch

        import stat
        fsync_original = os.fsync
        chamadas_fsync = 0

        def fsync_com_falha(fd):
            nonlocal chamadas_fsync
            if not stat.S_ISDIR(os.fstat(fd).st_mode):
                chamadas_fsync += 1
                if chamadas_fsync == numero_da_chamada:
                    raise OSError("fsync simulado")
            fsync_original(fd)

        with patch("ensaio_processo_atribuir.os.fsync", side_effect=fsync_com_falha):
            return self.rodar()

    def test_fsync_apos_post_atribuicao_conserva_efeito_incerto(self):
        codigo, relatorio = self.rodar_com_falha_fsync(2)
        self.assertEqual((codigo, relatorio["estado"]), (3, "EFEITO_INCERTO"))
        self.assertEqual(relatorio["residuo"], "possivel_atribuicao")
        self.assertEqual(relatorio["motivo"], "registro_indisponivel_apos_post")
        self.assertEqual(len(self.chamadas), 2)
        self.assertTrue(self.registro.exists())

    def test_fsync_apos_post_remocao_conserva_efeito_incerto(self):
        codigo, relatorio = self.rodar_com_falha_fsync(5)
        self.assertEqual((codigo, relatorio["estado"]), (3, "EFEITO_INCERTO"))
        self.assertEqual(relatorio["residuo"], "possivel_remocao")
        self.assertEqual(relatorio["motivo"], "registro_indisponivel_apos_post")
        self.assertEqual(len(self.chamadas), 5)
        self.assertTrue(self.registro.exists())

    def test_fsync_final_nao_publica_pass_indevido(self):
        codigo, relatorio = self.rodar_com_falha_fsync(6)
        self.assertEqual((codigo, relatorio["estado"]), (3, "EFEITO_INCERTO"))
        self.assertEqual(relatorio["residuo"], "possivel_remocao")
        self.assertEqual(relatorio["motivo"], "registro_indisponivel_apos_post")
        self.assertEqual(len(self.chamadas), 6)
        # fsync com falha não pode deixar um PASS no registro persistido.
        persistido = json.loads(self.registro.read_text())
        self.assertNotEqual(persistido["estado"], "PASS")
        self.assertNotEqual(persistido["residuo"], "sem_atribuicao")

    def test_trava_sincroniza_diretorio_antes_do_primeiro_post(self):
        import os
        import stat
        from unittest.mock import patch

        fsync_original = os.fsync
        diretorio_sincronizado = False
        respostas = iter(roteiro())

        def fsync_rastreado(fd):
            nonlocal diretorio_sincronizado
            if stat.S_ISDIR(os.fstat(fd).st_mode):
                diretorio_sincronizado = True
            return fsync_original(fd)

        def executar(_ies, argumentos):
            self.chamadas.append(argumentos)
            if "--confirmar" in argumentos:
                self.assertTrue(diretorio_sincronizado)
            return next(respostas)

        with patch("ensaio_processo_atribuir.os.fsync", side_effect=fsync_rastreado):
            codigo, relatorio = ensaiar(
                ies="ies-falso", registro=self.registro, nup=NUP,
                usuario=USUARIO, origem=ORIGEM, executar=executar,
                pausa=lambda _: None,
            )
        self.assertEqual((codigo, relatorio["estado"]), (0, "PASS"))
        self.assertEqual(len(self.chamadas), 6)

    def test_falha_fsync_diretorio_bloqueia_primeiro_post(self):
        import os
        import stat
        from unittest.mock import patch

        fsync_original = os.fsync

        def fsync_falhando_no_diretorio(fd):
            if stat.S_ISDIR(os.fstat(fd).st_mode):
                raise OSError("diretorio nao persistido")
            return fsync_original(fd)

        with patch("ensaio_processo_atribuir.os.fsync",
                   side_effect=fsync_falhando_no_diretorio):
            codigo, relatorio = self.rodar()
        self.assertEqual((codigo, relatorio["estado"]), (2, "BLOQUEADO"))
        self.assertEqual(len(self.chamadas), 1)
        self.assertTrue(self.registro.exists())

    def test_rename_final_sincroniza_diretorio_antes_de_pass(self):
        import os
        import stat
        from unittest.mock import patch

        fsync_original = os.fsync
        replace_original = os.replace
        houve_rename = False
        diretorio_apos_rename = False

        def substituir(*args):
            nonlocal houve_rename
            replace_original(*args)
            houve_rename = True

        def sincronizar(fd):
            nonlocal diretorio_apos_rename
            if stat.S_ISDIR(os.fstat(fd).st_mode) and houve_rename:
                diretorio_apos_rename = True
            return fsync_original(fd)

        with patch("ensaio_processo_atribuir.os.replace", side_effect=substituir), \
             patch("ensaio_processo_atribuir.os.fsync", side_effect=sincronizar):
            codigo, relatorio = self.rodar()
        self.assertEqual((codigo, relatorio["estado"]), (0, "PASS"))
        self.assertTrue(houve_rename)
        self.assertTrue(diretorio_apos_rename)

    def test_falha_fsync_diretorio_apos_rename_nao_declara_pass(self):
        import os
        import stat
        from unittest.mock import patch

        fsync_original = os.fsync
        chamadas_diretorio = 0

        def sincronizar(fd):
            nonlocal chamadas_diretorio
            if stat.S_ISDIR(os.fstat(fd).st_mode):
                chamadas_diretorio += 1
                if chamadas_diretorio == 2:
                    raise OSError("diretorio nao persistido apos rename")
            return fsync_original(fd)

        with patch("ensaio_processo_atribuir.os.fsync", side_effect=sincronizar):
            codigo, relatorio = self.rodar()
        self.assertEqual(chamadas_diretorio, 2)
        self.assertEqual((codigo, relatorio["estado"]), (3, "EFEITO_INCERTO"))
        self.assertEqual(relatorio["residuo"], "possivel_remocao")
        self.assertEqual(len(self.chamadas), 6)

    def test_escrita_parcial_do_checkpoint_antes_do_post_eh_completa(self):
        import os
        from unittest.mock import patch

        escrita_original = os.write
        parcial_injetada = False
        registro_antes_do_post = []
        respostas = iter(roteiro())

        def escrever_parcial(fd, dados):
            nonlocal parcial_injetada
            if not parcial_injetada:
                parcial_injetada = True
                return escrita_original(fd, dados[:1])
            return escrita_original(fd, dados)

        def executar(_ies, args):
            if "--confirmar" in args and not registro_antes_do_post:
                registro_antes_do_post.append(self.registro.read_text())
            return next(respostas)

        with patch("ensaio_processo_atribuir.os.write", side_effect=escrever_parcial):
            codigo, relatorio = ensaiar(
                ies="ies-falso", registro=self.registro, nup=NUP,
                usuario=USUARIO, origem=ORIGEM, executar=executar,
                pausa=lambda _: None,
            )
        self.assertTrue(parcial_injetada)
        self.assertEqual((codigo, relatorio["estado"]), (0, "PASS"))
        self.assertEqual(json.loads(registro_antes_do_post[0])["estado"],
                         "CONFIRMACAO_INICIADA")
        self.assertEqual(json.loads(self.registro.read_text())["estado"], "PASS")


class PrevooDaOrigem(unittest.TestCase):
    def test_tabela_com_crases_reais_contem_sete_origens(self):
        from ensaio_processo_atribuir import origens_de_treinamento
        tabela = (Path(__file__).resolve().parents[1] /
                  "docs/interfaces/credenciais-de-treinamento.md")
        origens = origens_de_treinamento(tabela)
        self.assertEqual(len(origens), 7)
        self.assertIn(ORIGEM, origens)
        self.assertNotIn("https://sei.sistemas.ro.gov.br/sei", origens)

    def test_origem_privada_exata_e_configuracao_paralela(self):
        import os
        from unittest.mock import patch
        from ensaio_processo_atribuir import origem_privada_permitida
        with tempfile.TemporaryDirectory() as pasta:
            arquivo = Path(pasta) / "origem"
            arquivo.write_text(ORIGEM, encoding="utf-8")
            arquivo.chmod(0o600)
            with patch.dict(os.environ, {"IES_ORIGEM_FILE": str(arquivo)}, clear=True):
                self.assertEqual(origem_privada_permitida({ORIGEM}), ORIGEM)
            with patch.dict(os.environ, {"IES_ORIGEM_FILE": str(arquivo),
                                        "IES_CONFIG_FILE": "/config-concorrente.json"},
                            clear=True):
                with self.assertRaises(ValueError):
                    origem_privada_permitida({ORIGEM})
