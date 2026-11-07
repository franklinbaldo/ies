"""Testes sintéticos do ensaio #245: nenhuma rede, login ou escrita."""
import json
import os
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from scripts.ensaio_processo_criar import ensaiar, origens_de_treinamento, origem_privada_permitida

ORIGEM = "https://sei.treinamento.unb.br/sei"
NUP = "00000.000001/2026-00"


class EnsaioUmaVez(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.registro = Path(self.tmp.name) / "execucao.json"
        self.chamadas = []
        self.pausas = []

    def rodar(self, respostas):
        fila = iter(respostas)

        def executar(_ies, argumentos):
            self.chamadas.append(argumentos)
            return next(fila)

        return ensaiar(
            ies="binario-falso", registro=self.registro, tipo="Ensaio",
            especificacao="Teste sintético", nivel="publico", hipotese=None,
            origem=ORIGEM, executar=executar, pausa=self.pausas.append,
        )

    def respostas_validas(self):
        return [
            (0, {"tipo": "Ensaio", "especificacao": "Teste sintético", "nivel": "publico", "unidade": "TREINO", "confirmado": False, "requisicoes": 3}),
            (0, {"tipo": "Ensaio", "especificacao": "Teste sintético", "nivel": "publico", "unidade": "TREINO", "confirmado": True, "nup": NUP, "requisicoes": 7}),
            (0, {"numero": NUP, "id_procedimento": "10", "requisicoes": 2}),
        ]

    def test_ordem_confirmacao_unica_releitura_identidade_e_registro_privado(self):
        codigo, relatorio = self.rodar(self.respostas_validas())
        self.assertEqual(codigo, 0)
        self.assertEqual(relatorio["estado"], "PASS")
        self.assertEqual([c[-1] == "--confirmar" for c in self.chamadas], [False, True, False])
        self.assertEqual(self.chamadas[-1], ["processo", NUP])
        self.assertEqual(self.pausas, [30, 30])
        self.assertEqual(self.registro.stat().st_mode & 0o777, 0o600)
        self.assertEqual(json.loads(self.registro.read_text())["nup"], NUP)
        self.assertEqual(relatorio["requisicoes_previa"], 3)
        self.assertEqual(relatorio["requisicoes_confirmacao"], 7)
        self.assertEqual(relatorio["requisicoes_verificacao"], 2)

    def test_previa_falhou_sem_trava_nem_post(self):
        codigo, relatorio = self.rodar([(1, {"subcodigo": "tipo_nao_encontrado"})])
        self.assertEqual((codigo, relatorio["estado"]), (1, "PREVIA_FAIL"))
        self.assertEqual(len(self.chamadas), 1)
        self.assertFalse(self.registro.exists())

    def test_previa_precisa_confirmado_false_e_tipo_exato(self):
        respostas = self.respostas_validas()
        respostas[0][1]["tipo"] = "Outro"
        codigo, _ = self.rodar(respostas)
        self.assertEqual(codigo, 1)
        self.assertEqual(len(self.chamadas), 1)
        self.assertFalse(self.registro.exists())

    def test_previa_sem_unidade_observada_recusa_sem_post(self):
        respostas = self.respostas_validas()
        respostas[0][1].pop("unidade")
        codigo, relatorio = self.rodar(respostas)
        self.assertEqual((codigo, relatorio["estado"]), (1, "PREVIA_FAIL"))
        self.assertEqual(len(self.chamadas), 1)
        self.assertFalse(self.registro.exists())

    def test_confirmacao_de_outro_tipo_nao_prova_o_post(self):
        respostas = self.respostas_validas()
        respostas[1][1]["tipo"] = "Outro tipo"
        codigo, relatorio = self.rodar(respostas)
        self.assertEqual((codigo, relatorio["estado"]), (3, "CONFIRMACAO_DIVERGENTE"))
        self.assertEqual(relatorio["nup"], NUP)
        self.assertEqual(len(self.chamadas), 3)  # releitura independente preservada
        self.assertEqual(self.chamadas[-1], ["processo", NUP])
        self.assertEqual(json.loads(self.registro.read_text())["estado"], "CONFIRMACAO_DIVERGENTE")

    def test_confirmacao_sem_especificacao_nao_prova_o_post(self):
        respostas = self.respostas_validas()
        respostas[1][1].pop("especificacao")
        codigo, relatorio = self.rodar(respostas)
        self.assertEqual((codigo, relatorio["estado"]), (3, "CONFIRMACAO_DIVERGENTE"))
        self.assertEqual(relatorio["nup"], NUP)
        self.assertEqual(len(self.chamadas), 3)

    def test_confirmacao_em_outra_unidade_nao_prova_o_post(self):
        respostas = self.respostas_validas()
        respostas[1][1]["unidade"] = "OUTRA_UNIDADE"
        codigo, relatorio = self.rodar(respostas)
        self.assertEqual((codigo, relatorio["estado"]), (3, "CONFIRMACAO_DIVERGENTE"))
        self.assertEqual(relatorio["nup"], NUP)
        self.assertEqual(len(self.chamadas), 3)

    def test_segunda_execucao_recusa_antes_da_rede(self):
        self.assertEqual(self.rodar(self.respostas_validas())[0], 0)
        self.chamadas.clear()
        codigo, relatorio = self.rodar(self.respostas_validas())
        self.assertEqual((codigo, relatorio["estado"]), (2, "BLOQUEADO"))
        self.assertEqual(self.chamadas, [])

    def test_confirmacao_incertidao_nao_repete_nem_releitura(self):
        respostas = self.respostas_validas()
        respostas[1] = (3, {"subcodigo": "efeito_incerto_na_criacao"})
        codigo, relatorio = self.rodar(respostas)
        self.assertEqual((codigo, relatorio["estado"]), (3, "EFEITO_INCERTO"))
        self.assertEqual(len(self.chamadas), 2)
        self.assertTrue(self.registro.exists())
        self.assertEqual(json.loads(self.registro.read_text())["estado"], "EFEITO_INCERTO")

    def test_confirmacao_com_orcamento_incoerente_nao_declara_pass(self):
        respostas = self.respostas_validas()
        respostas[1][1]["requisicoes"] = 5
        codigo, relatorio = self.rodar(respostas)
        self.assertEqual((codigo, relatorio["estado"]), (3, "ORCAMENTO_DIVERGENTE"))
        self.assertEqual(len(self.chamadas), 3)

    def test_orcamento_sem_terceira_leitura_verificadora_nao_declara_pass(self):
        # Criador: 1 POST; prova independente: tela, arvore, propriedades (3 GET).
        # O total antigo previa+3 nao comprova o fluxo atual da #245.
        respostas = self.respostas_validas()
        respostas[1][1]["requisicoes"] = 6
        codigo, relatorio = self.rodar(respostas)
        self.assertEqual((codigo, relatorio["estado"]), (3, "ORCAMENTO_DIVERGENTE"))
        self.assertEqual(len(self.chamadas), 3)
        self.assertEqual(relatorio["requisicoes_confirmacao"], 6)

    def test_releitura_sem_contagem_nao_declara_pass(self):
        respostas = self.respostas_validas()
        respostas[2][1].pop("requisicoes")
        codigo, relatorio = self.rodar(respostas)
        self.assertEqual((codigo, relatorio["estado"]), (3, "VERIFICACAO_INDEPENDENTE_FALHOU"))
        self.assertEqual(len(self.chamadas), 3)

    def test_releitura_outra_identidade_nao_aceita_pass(self):
        respostas = self.respostas_validas()
        respostas[2] = (0, {"numero": "99999.000001/2026-00", "id_procedimento": "10"})
        codigo, relatorio = self.rodar(respostas)
        self.assertEqual((codigo, relatorio["estado"]), (3, "VERIFICACAO_INDEPENDENTE_FALHOU"))
        self.assertEqual(len(self.chamadas), 3)
        self.assertEqual(relatorio["nup"], NUP)

    def test_releitura_sem_id_nao_aceita_pass(self):
        respostas = self.respostas_validas()
        respostas[2] = (0, {"numero": NUP, "id_procedimento": None})
        self.assertEqual(self.rodar(respostas)[0], 3)

    def test_timeout_apos_inicio_eh_incerto_sem_retry(self):
        def executar(_ies, args):
            self.chamadas.append(args)
            if len(self.chamadas) == 1:
                return self.respostas_validas()[0]
            raise OSError("timeout sintético")

        codigo, relatorio = ensaiar(
            ies="binario-falso", registro=self.registro, tipo="Ensaio",
            especificacao="Teste sintético", nivel="publico", hipotese=None,
            origem=ORIGEM, executar=executar, pausa=lambda _: None,
        )
        self.assertEqual((codigo, relatorio["estado"]), (3, "EFEITO_INCERTO"))
        self.assertEqual(len(self.chamadas), 2)

    def test_wip_245_origem_sei_com_versao_declarada_eh_permitida(self):
        # O leitor IES aceita segunda linha "versao=4.0.9" na origem 0600.
        # O ensaio deve autorizar essa mesma instalacao de treinamento.
        origem = Path(self.tmp.name) / "origem-com-versao"
        origem.write_text(ORIGEM + "\nversao=4.0.9\n")
        origem.chmod(0o600)
        with patch.dict(os.environ, {
            "IES_ORIGEM_FILE": str(origem), "IES_CONFIG_FILE": "",
        }):
            self.assertEqual(origem_privada_permitida({ORIGEM}), ORIGEM)

    def test_wip_245_metadado_invalido_ou_linha_extra_recusada(self):
        origem = Path(self.tmp.name) / "origem-com-metadado"
        with patch.dict(os.environ, {
            "IES_ORIGEM_FILE": str(origem), "IES_CONFIG_FILE": "",
        }):
            for conteudo in (
                ORIGEM + "\nversao=4.x\n",
                ORIGEM + "\nversao=4.0.9\nlinha-adicional\n",
                ORIGEM + "\noutro=4.0.9\n",
            ):
                with self.subTest(conteudo=conteudo):
                    origem.write_text(conteudo)
                    origem.chmod(0o600)
                    with self.assertRaises(ValueError):
                        origem_privada_permitida({ORIGEM})

    def test_configuracao_local_nao_pode_substituir_origem_permitida(self):
        origem = Path(self.tmp.name) / "origem"
        origem.write_text(ORIGEM + "\n")
        origem.chmod(0o600)
        with patch.dict(os.environ, {
            "IES_ORIGEM_FILE": str(origem),
            "IES_CONFIG_FILE": "/nao/deve/substituir/sessao.json",
        }):
            with self.assertRaises(ValueError):
                origem_privada_permitida({ORIGEM})

    def test_origens_exigem_sete_ambientes_explicitos(self):
        f = Path(self.tmp.name) / "tabela.md"
        f.write_text("| " + chr(96) + "unb" + chr(96)
                     + " | UnB | https://sei.treinamento.unb.br/sei | user | pass | org | fonte |\n")
        with self.assertRaises(ValueError):
            origens_de_treinamento(f)


    def test_previa_divergente_na_especificacao_recusa_antes_da_escrita(self):
        respostas = self.respostas_validas()
        respostas[0][1]["especificacao"] = "Especificação diferente da solicitada"
        codigo, relatorio = self.rodar(respostas)
        self.assertEqual((codigo, relatorio["estado"]), (1, "PREVIA_FAIL"))
        self.assertEqual(len(self.chamadas), 1)
        self.assertFalse(self.registro.exists())

    def test_previa_sem_especificacao_observada_recusa_antes_da_escrita(self):
        respostas = self.respostas_validas()
        respostas[0][1].pop("especificacao", None)
        codigo, relatorio = self.rodar(respostas)
        self.assertEqual((codigo, relatorio["estado"]), (1, "PREVIA_FAIL"))
        self.assertEqual(len(self.chamadas), 1)
        self.assertFalse(self.registro.exists())

    def test_contagem_booleana_nao_autoriza_escrita(self):
        respostas = self.respostas_validas()
        respostas[0][1]["requisicoes"] = True
        codigo, relatorio = self.rodar(respostas)
        self.assertEqual((codigo, relatorio["estado"]), (1, "PREVIA_FAIL"))
        self.assertEqual(len(self.chamadas), 1)
        self.assertFalse(self.registro.exists())


    def test_contagem_de_previa_nao_reflete_url_na_evidencia(self):
        respostas = self.respostas_validas()
        respostas[0][1]["requisicoes"] = "https://example.invalid/nao-publicar"
        codigo, relatorio = self.rodar(respostas)
        self.assertEqual((codigo, relatorio["estado"]), (1, "PREVIA_FAIL"))
        self.assertIsNone(relatorio["requisicoes_previa"])
        self.assertEqual(len(self.chamadas), 1)

    def test_contagem_de_confirmacao_nao_reflete_url_no_registro(self):
        respostas = self.respostas_validas()
        respostas[1] = (3, {"requisicoes": "https://example.invalid/nao-publicar"})
        codigo, relatorio = self.rodar(respostas)
        self.assertEqual((codigo, relatorio["estado"]), (3, "EFEITO_INCERTO"))
        self.assertIsNone(relatorio["requisicoes_confirmacao"])
        self.assertNotIn("example.invalid", self.registro.read_text())
        self.assertEqual(len(self.chamadas), 2)

    def test_contagem_de_releitura_nao_reflete_url_no_registro(self):
        respostas = self.respostas_validas()
        respostas[2] = (1, {"requisicoes": "https://example.invalid/nao-publicar"})
        codigo, relatorio = self.rodar(respostas)
        self.assertEqual((codigo, relatorio["estado"]), (3, "VERIFICACAO_INDEPENDENTE_FALHOU"))
        self.assertIsNone(relatorio["requisicoes_verificacao"])
        self.assertNotIn("example.invalid", self.registro.read_text())
        self.assertEqual(len(self.chamadas), 3)


    def test_post_inconclusivo_preserva_causa_e_nup_sem_prova_causal(self):
        # JSON emitido pela CLI ao encontrar um unico candidato depois de POST incerto.
        respostas = self.respostas_validas()
        respostas[1] = (3, {"codigo": "efeito_incerto",
                            "subcodigo": "candidato_unico_sem_prova_causal",
                            "causa_post": "falha_de_rede",
                            "candidatos_compativeis": [NUP],
                            "requisicoes": 8})
        codigo, relatorio = self.rodar(respostas)
        self.assertEqual((codigo, relatorio["estado"]), (3, "EFEITO_INCERTO"))
        self.assertEqual(relatorio["subcodigo_post"], "candidato_unico_sem_prova_causal")
        self.assertEqual(relatorio["causa_post"], "falha_de_rede")
        self.assertEqual(relatorio["possiveis_residuos"], [NUP])
        self.assertEqual(relatorio["requisicoes_confirmacao"], 8)
        self.assertEqual(json.loads(self.registro.read_text())["possiveis_residuos"], [NUP])
        self.assertEqual(len(self.chamadas), 2)  # jamais tenta novamente nem relê como prova causal


    def test_diagnostico_de_post_inconclusivo_nao_republica_url_assinada(self):
        respostas = self.respostas_validas()
        url = "https://example.invalid/?infra_hash=segredo"
        respostas[1] = (3, {"codigo": "efeito_incerto",
                            "subcodigo": url,
                            "causa_post": "falha_de_rede\\n" + url,
                            "candidatos_compatíveis": [url],
                            "candidatos_compativeis": [NUP, url],
                            "requisicoes": 8})
        codigo, relatorio = self.rodar(respostas)
        self.assertEqual((codigo, relatorio["estado"]), (3, "EFEITO_INCERTO"))
        self.assertEqual(relatorio["possiveis_residuos"], [NUP])
        self.assertNotIn("subcodigo_post", relatorio)
        self.assertNotIn("causa_post", relatorio)
        self.assertNotIn("infra_hash", self.registro.read_text())
        self.assertEqual(len(self.chamadas), 2)


if __name__ == "__main__":
    unittest.main()
