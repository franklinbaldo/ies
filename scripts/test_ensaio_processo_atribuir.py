"""Regressões offline do ensaio único de atribuição #264 (zero rede)."""
import json
import tempfile
import unittest
from pathlib import Path

from ensaio_processo_atribuir import ORIGEM_UNB, conta_unb, ensaiar

NUP = "23106.001559/2026-11"
USUARIO = "aluno2"
DOC = Path(__file__).resolve().parents[1] / "docs/interfaces/credenciais-de-treinamento.md"


def resposta(*, atual, pedido, confirmado, requisicoes):
    return (0, {
        "nup": NUP, "id_procedimento": "777", "unidade": "UNB/TESTES",
        "usuario_atual": atual, "usuario_pedido": pedido,
        "confirmado": confirmado, "alterado": confirmado,
        "requisicoes": requisicoes,
    })


class EnsaioUnico(unittest.TestCase):
    def setUp(self):
        t = tempfile.TemporaryDirectory()
        self.addCleanup(t.cleanup)
        self.registro = Path(t.name) / "tentativa.json"
        self.comandos = []
        self.pausas = []

    def rodar(self, respostas, *, origem=ORIGEM_UNB):
        fila = iter(respostas)

        def executar(_ies, args):
            self.comandos.append(args)
            return next(fila)

        return ensaiar(
            ies="ies-falso", nup=NUP, usuario=USUARIO, registro=self.registro,
            origem=origem, executar=executar, pausa=self.pausas.append,
        )

    def respostas_validas(self):
        return [
            resposta(atual="", pedido=USUARIO, confirmado=False, requisicoes=4),
            resposta(atual="", pedido=USUARIO, confirmado=True, requisicoes=6),
            resposta(atual=USUARIO, pedido=None, confirmado=False, requisicoes=4),
            resposta(atual=USUARIO, pedido=None, confirmado=True, requisicoes=6),
            resposta(atual="", pedido=USUARIO, confirmado=False, requisicoes=4),
        ]

    def test_ciclo_unico_com_prova_e_registro_privado(self):
        codigo, relatorio = self.rodar(self.respostas_validas())
        self.assertEqual((codigo, relatorio["estado"]), (0, "PASS"))
        self.assertEqual(len(self.comandos), 5)
        self.assertEqual([c.count("--confirmar") for c in self.comandos], [0, 1, 0, 1, 0])
        self.assertEqual(["--remover" in c for c in self.comandos], [False, False, True, True, False])
        self.assertEqual(self.pausas, [35] * 4)
        self.assertEqual(self.registro.stat().st_mode & 0o777, 0o600)
        salvo = json.loads(self.registro.read_text(encoding="utf-8"))
        self.assertEqual(salvo["estado"], "PASS")
        self.assertEqual(salvo["requisicoes"], [4, 6, 4, 6, 4])

    def test_previa_falhou_sem_nenhuma_escrita(self):
        codigo, relatorio = self.rodar([(1, {"subcodigo": "usuario_nao_atribuivel"})])
        self.assertEqual((codigo, relatorio["estado"]), (1, "PREVIA_FAIL"))
        self.assertEqual(len(self.comandos), 1)
        self.assertTrue(self.registro.exists())

    def test_confirmacao_incerta_nao_tenta_remocao_nem_retry(self):
        respostas = self.respostas_validas()
        respostas[1] = (3, {"subcodigo": "efeito_incerto_na_atribuicao", "requisicoes": 6})
        codigo, relatorio = self.rodar(respostas)
        self.assertEqual((codigo, relatorio["estado"]), (3, "EFEITO_INCERTO"))
        self.assertEqual(len(self.comandos), 2)
        self.assertEqual(self.rodar(self.respostas_validas())[0], 2)
        self.assertEqual(len(self.comandos), 2)

    def test_previa_nao_autoriza_post_se_unidade_ou_atribuicao_divergir(self):
        respostas = self.respostas_validas()
        respostas[0][1]["usuario_atual"] = "outra_pessoa"
        codigo, relatorio = self.rodar(respostas)
        self.assertEqual((codigo, relatorio["estado"]), (1, "PREVIA_FAIL"))
        self.assertEqual(len(self.comandos), 1)

    def test_releitura_divergente_nao_tenta_remocao(self):
        respostas = self.respostas_validas()
        respostas[2][1]["id_procedimento"] = "888"
        codigo, relatorio = self.rodar(respostas)
        self.assertEqual((codigo, relatorio["estado"]), (3, "RELEITURA_DIVERGENTE"))
        self.assertEqual(len(self.comandos), 3)

    def test_remocao_incerta_nao_repete_post_e_registra_residuo(self):
        respostas = self.respostas_validas()
        respostas[3] = (3, {"subcodigo": "efeito_incerto_na_atribuicao", "requisicoes": 6})
        codigo, relatorio = self.rodar(respostas)
        self.assertEqual((codigo, relatorio["estado"]), (3, "EFEITO_INCERTO"))
        self.assertEqual(len(self.comandos), 4)
        self.assertEqual(relatorio["passo"], "confirmar_remocao")

    def test_origem_nao_treinamento_bloqueia_sem_comandos(self):
        codigo, relatorio = self.rodar(self.respostas_validas(), origem="https://sei.exemplo.gov.br/sei")
        self.assertEqual((codigo, relatorio["estado"]), (2, "BLOQUEADO"))
        self.assertEqual(self.comandos, [])

    def test_contagem_nao_numerica_nunca_vira_pass(self):
        respostas = self.respostas_validas()
        respostas[1][1]["requisicoes"] = True
        codigo, relatorio = self.rodar(respostas)
        self.assertEqual((codigo, relatorio["estado"]), (3, "EFEITO_INCERTO"))
        self.assertEqual(len(self.comandos), 2)

    def test_parser_de_credenciais_reais_aceita_crases_mas_nao_escapes(self):
        origem, usuario, senha, orgao = conta_unb(DOC)
        self.assertEqual((origem, usuario, orgao), (ORIGEM_UNB, "aluno1", "UnB"))
        self.assertTrue(senha)

    def test_parser_recusa_duplicidade_ou_origem_incompativel(self):
        with tempfile.TemporaryDirectory() as d:
            path = Path(d) / "contas.md"
            row = f"| `unb` | UnB | {ORIGEM_UNB} | `aluno1` | `p` | `UnB` | [fonte](https://example.org) |\n"
            path.write_text(row * 2, encoding="utf-8")
            with self.assertRaises(ValueError):
                conta_unb(path)
            path.write_text(row.replace(ORIGEM_UNB, "https://sei.producao.gov.br/sei"), encoding="utf-8")
            with self.assertRaises(ValueError):
                conta_unb(path)


if __name__ == "__main__":
    unittest.main()
