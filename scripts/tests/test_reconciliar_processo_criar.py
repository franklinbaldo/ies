"""Cenários de reconciliação #245; dados sintéticos e nenhuma chamada real."""
import unittest
from scripts.reconciliar_processo_criar import reconciliar

ORIGEM = "TESTE"
NUP = "00123.000001/2026-00"


def lista(numeros, total=None, **extra):
    return {"categoria": "gerados", "unidade": ORIGEM,
            "total_declarado": len(numeros) if total is None else total,
            "faixa_observada": {"inicio": 1, "fim": len(numeros)} if numeros else None,
            "processos": [{"numero": n} for n in numeros], "requisicoes": 1, **extra}


def processo(nup=NUP, descricao="Ensaio IES 245 - escrita unica em treinamento", **extra):
    return {"numero": nup, "unidade": ORIGEM, "id_procedimento": "123",
            "propriedades": {"especificacao": descricao}, "requisicoes": 3, **extra}


class ConciliacaoSomenteLeitura(unittest.TestCase):
    def setUp(self):
        self.pedidos = []
        self.pausas = []

    def rodar(self, respostas, **kwargs):
        fila = iter(respostas)
        def executar(etapa, args):
            self.pedidos.append((etapa, args))
            return next(fila)
        return reconciliar(executar, self.pausas.append, **kwargs)

    def test_encontra_residuo_possivel_com_identidade_e_contagem(self):
        r = self.rodar([(0, lista([NUP])), (0, processo())])
        self.assertEqual(r["estado"], "POSSIVEL_RESIDUO")
        self.assertEqual(r["requisicoes_totais_observadas"], 4)
        self.assertEqual(r["correspondencias"], [{"nup": NUP, "id_procedimento": "123"}])
        self.assertEqual(r["processos_examinados"], 1)
        self.assertTrue(r["cobertura_total"])
        self.assertEqual(self.pausas, [35])
        self.assertEqual(self.pedidos[0][1], ["unidade", "gerados"])
        self.assertEqual(self.pedidos[1][1], ["processo", NUP,
                         "--sem-documentos", "--sem-marcadores", "--sem-andamentos"])
        self.assertFalse(any("--confirmar" in x for _, args in self.pedidos for x in args))

    def test_nao_achou_mais_pagina_pendente_nunca_alega_sem_residuo(self):
        r = self.rodar([(0, lista([NUP], total=400)), (0, processo(descricao="Outro"))])
        self.assertEqual(r["estado"], "INCONCLUSIVO")
        self.assertFalse(r["cobertura_total"])
        self.assertEqual(r["total_declarado"], 400)
        self.assertEqual(r["correspondencias"], [])

    def test_formato_de_gerados_nao_confiavel_aborta_antes_de_abrir_processo(self):
        d = lista([NUP]); d["categoria"] = "recebidos"
        r = self.rodar([(0, d)])
        self.assertEqual(r["estado"], "FORMATO_NAO_RECONHECIDO")
        self.assertEqual(len(self.pedidos), 1)

    def test_paginas_ou_faixa_nao_verificadas_nao_sao_tratadas_como_primeira(self):
        d = lista([NUP]); d["faixa_observada"] = {"inicio": 11, "fim": 11}
        self.assertEqual(self.rodar([(0, d)])["estado"], "FORMATO_NAO_RECONHECIDO")
        self.assertEqual(len(self.pedidos), 1)

    def test_numero_com_url_ou_controle_nao_vira_argumento(self):
        r = self.rodar([(0, lista(["https://evil.invalid/x", "A\nB"]))])
        self.assertEqual(r["estado"], "INCONCLUSIVO")
        self.assertEqual(len(self.pedidos), 1)

    def test_erro_do_processo_para_sem_retry(self):
        r = self.rodar([(0, lista([NUP, "00123.000002/2026-00"])),
                      (1, {"subcodigo": "transporte_falhou", "requisicoes": 1})])
        self.assertEqual(r["estado"], "LEITURA_INTERROMPIDA")
        self.assertEqual(len(self.pedidos), 2)
        self.assertEqual(r["requisicoes_totais_observadas"], 2)

    def test_identidade_e_unidade_distintas_nao_sao_correspondencia(self):
        r = self.rodar([(0, lista([NUP])), (0, processo(unidade="OUTRA"))])
        self.assertEqual(r["estado"], "INCONCLUSIVO")

    def test_falta_contagem_nao_esta_zero(self):
        d = processo(); d.pop("requisicoes")
        r = self.rodar([(0, lista([NUP])), (0, d)])
        self.assertIsNone(r["requisicoes_totais_observadas"])

    def test_dois_registros_iguais_na_lista_so_uma_leitura(self):
        r = self.rodar([(0, lista([NUP, NUP])), (0, processo())])
        self.assertEqual(r["estado"], "POSSIVEL_RESIDUO")
        self.assertEqual(len(self.pedidos), 2)

    def test_limite_contem_efeitos_de_leitura_marca_visualizado(self):
        numeros = [f"00123.{i:06d}/2026-00" for i in range(1, 20)]
        r = self.rodar([(0, lista(numeros, total=500))]
                     + [(0, processo(nup=n, descricao="Outro")) for n in numeros[:12]])
        self.assertEqual(r["processos_examinados"], 12)
        self.assertEqual(len(self.pedidos), 13)
        self.assertFalse(r["cobertura_total"])
        self.assertEqual(len(self.pausas), 12)

    def test_subcodigo_de_resposta_nao_pode_vazar_url_no_log(self):
        r = self.rodar([(1, {"subcodigo": "https://example.invalid/?infra_hash=segredo",
                             "requisicoes": -1})])
        self.assertEqual(r["estado"], "LEITURA_INTERROMPIDA")
        self.assertIsNone(r["etapas"][0]["subcodigo"])
        self.assertIsNone(r["etapas"][0]["requisicoes"])
        self.assertIsNone(r["requisicoes_totais_observadas"])

    def test_parametros_inseguros_bloqueados_antes_da_rede(self):
        with self.assertRaises(ValueError):
            self.rodar([], intervalo=0)
        with self.assertRaises(ValueError):
            self.rodar([], limite=100)
        self.assertEqual(len(self.pedidos), 0)


if __name__ == '__main__':
    unittest.main()
