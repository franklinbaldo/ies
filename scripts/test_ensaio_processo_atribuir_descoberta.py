"""Testes tecnicos da descoberta somente-leitura do ensaio #264, sem SEI."""
import unittest

import ensaio_processo_atribuir_descoberta as raposa


ORIGEM = "https://sei.treinamento.unb.br/sei"
NUP1 = "23106.000101/2026-01"
NUP2 = "23106.000102/2026-02"


def caixa():
    return {"categoria": "recebidos", "unidade": "TESTE", "processos": [
        {"id": "41", "numero": NUP1},
        {"id": "42", "numero": NUP2},
    ]}


def processo(nup, ident, unidade, atribuicao, requisicoes=2):
    return {"numero": nup, "id_procedimento": ident, "requisicoes": requisicoes,
            "aberto_em": [{"sigla": unidade, "atribuido_a": atribuicao}]}


class DescobertaSemEscrita(unittest.TestCase):
    def rodar(self, respostas, limite=3):
        chamadas, pausas = [], []
        iterador = iter(respostas)

        def executar(_ies, args):
            chamadas.append(args)
            return next(iterador)

        codigo, resumo = raposa.descobrir_candidato(
            ies="ies-falso", origem=ORIGEM, executar=executar,
            pausa=pausas.append, limite=limite,
        )
        return codigo, resumo, chamadas, pausas

    def test_observa_candidato_da_unidade_ativa_e_nao_confunde_outra_unidade(self):
        respostas = [(0, caixa()),
                     (0, processo(NUP1, "41", "OUTRA", None)),
                     (0, processo(NUP2, "42", "TESTE", None))]
        codigo, resumo, chamadas, pausas = self.rodar(respostas)
        self.assertEqual((codigo, resumo["estado"]), (0, "CANDIDATO_OBSERVADO"))
        self.assertEqual(resumo["nup"], NUP2)
        self.assertEqual(resumo["consultas_processos"], 2)
        self.assertEqual(resumo["requisicoes"], [None, 2, 2])
        self.assertEqual(chamadas, [
            ["unidade", "recebidos"],
            ["processo", NUP1, "--sem-documentos", "--sem-andamentos",
             "--sem-marcadores", "--sem-propriedades"],
            ["processo", NUP2, "--sem-documentos", "--sem-andamentos",
             "--sem-marcadores", "--sem-propriedades"],
        ])
        self.assertEqual(pausas, [30, 30])
        self.assertTrue(all("--confirmar" not in args for args in chamadas))

    def test_nao_presume_ausencia_quando_atribuicao_nao_eh_publicada(self):
        sem_atribuicao_observavel = processo(NUP1, "41", "TESTE", None)
        del sem_atribuicao_observavel["aberto_em"][0]["atribuido_a"]
        codigo, resumo, chamadas, _ = self.rodar([
            (0, caixa()), (0, sem_atribuicao_observavel),
            (0, processo(NUP2, "42", "TESTE", {"sigla": "outro"})),
        ])
        self.assertEqual((codigo, resumo["estado"]), (1, "SEM_CANDIDATO_OBSERVADO"))
        self.assertNotIn("nup", resumo)
        self.assertEqual(len(chamadas), 3)

    def test_identidade_divergente_nao_autoriza_candidato(self):
        codigo, resumo, chamadas, _ = self.rodar([
            (0, caixa()), (0, processo(NUP1, "999", "TESTE", None)),
            (0, processo(NUP2, "42", "TESTE", {"sigla": "outro"})),
        ])
        self.assertEqual((codigo, resumo["estado"]), (1, "SEM_CANDIDATO_OBSERVADO"))
        self.assertNotIn("nup", resumo)
        self.assertEqual(len(chamadas), 3)

    def test_erro_de_leitura_interrompe_sem_tentar_outro_processo(self):
        codigo, resumo, chamadas, _ = self.rodar([
            (0, caixa()), (1, {"subcodigo": "pagina_de_login_recebida_no_controle",
                                "requisicoes": 2}),
        ])
        self.assertEqual((codigo, resumo["estado"]), (1, "LEITURA_FAIL"))
        self.assertEqual(len(chamadas), 2)
        self.assertNotIn("nup", resumo)

    def test_limita_observacoes_sem_paginar_ou_presumir_inexistencia(self):
        lista = caixa()
        lista["processos"] = [
            {"id": str(i), "numero": f"23106.{i:06d}/2026-01"}
            for i in range(1, 9)
        ]
        respostas = [(0, lista)] + [
            (0, processo(f"23106.{i:06d}/2026-01", str(i), "TESTE",
                         {"sigla": "outro"}))
            for i in range(1, 4)
        ]
        codigo, resumo, chamadas, _ = self.rodar(respostas)
        self.assertEqual((codigo, resumo["estado"]), (1, "SEM_CANDIDATO_OBSERVADO"))
        self.assertEqual(resumo["consultas_processos"], 3)
        self.assertEqual(len(chamadas), 4)
        self.assertNotIn("nup", resumo)

    def test_caixa_sem_unidade_ou_categoria_verificavel_nao_consulta_nup(self):
        for alteracao in ({"unidade": None}, {"categoria": None}):
            with self.subTest(alteracao=alteracao):
                lista = {**caixa(), **alteracao}
                codigo, resumo, chamadas, _ = self.rodar([(0, lista)])
                self.assertEqual((codigo, resumo["estado"]), (1, "LEITURA_FAIL"))
                self.assertEqual(chamadas, [["unidade", "recebidos"]])
                self.assertNotIn("nup", resumo)
