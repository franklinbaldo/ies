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
