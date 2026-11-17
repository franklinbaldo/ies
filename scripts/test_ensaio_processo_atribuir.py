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

class BaseEnsaio:

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


class EnsaioAtribuicao(BaseEnsaio, unittest.TestCase):
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

    def test_previa_falhou_preserva_diagnostico_tipado_sem_dados_livres(self):
        codigo, relatorio = self.rodar([(
            1, {"subcodigo": "situacao_da_atribuicao_nao_observada",
                "status_http": 403, "requisicoes": 3,
                "mensagem_do_servidor": "https://example.invalid/privado"}
        )])
        self.assertEqual(codigo, 1)
        self.assertEqual(relatorio["estado"], "PREVIA_FAIL")
        self.assertEqual(relatorio["requisicoes"], [3])
        self.assertEqual(relatorio["saida_cli"], 1)
        self.assertEqual(relatorio["subcodigo"],
                         "situacao_da_atribuicao_nao_observada")
        self.assertEqual(relatorio["status_http"], 403)
        self.assertNotIn("mensagem_do_servidor", relatorio)
        self.assertNotIn("example.invalid", json.dumps(relatorio))
        self.assertFalse(self.registro.exists())

    def test_previa_nao_publica_codigo_textual_arbitrario(self):
        codigo, relatorio = self.rodar([(
            1, {"subcodigo": "https://example.invalid/privado",
                "status_http": "https://example.invalid", "requisicoes": None}
        )])
        self.assertEqual((codigo, relatorio["estado"]), (1, "PREVIA_FAIL"))
        self.assertNotIn("subcodigo", relatorio)
        self.assertNotIn("status_http", relatorio)
        self.assertFalse(self.registro.exists())

    def test_previa_falhou_preserva_somente_confronto_de_unidades(self):
        # O proprio JSON tipado da CLI publica a unidade da tela e as
        # unidades abertas. Comparar sem publicar os nomes no resumo.
        for unidade, aberto_em, esperado in (
            ("Unidade2", ["Unidade1"], False),
            ("Unidade1", ["Unidade1"], True),
            ("Unidade1", None, None),
        ):
            with self.subTest(unidade=unidade, aberto_em=aberto_em):
                codigo, relatorio = self.rodar([(
                    1, {"subcodigo": "situacao_da_atribuicao_nao_observada",
                        "unidade": unidade, "aberto_em": aberto_em,
                        "requisicoes": 3}
                )])
                self.assertEqual((codigo, relatorio["estado"]), (1, "PREVIA_FAIL"))
                self.assertEqual(relatorio.get("unidade_consta_na_arvore"), esperado)
                self.assertNotIn("unidade", relatorio)
                self.assertNotIn("aberto_em", relatorio)
                self.assertFalse(self.registro.exists())
                self.chamadas.clear()

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
