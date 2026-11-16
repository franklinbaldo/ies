# /// script
# requires-python = ">=3.11"
# dependencies = [
#     "keyring>=24.0.0",
#     "rich>=13.0.0",
# ]
# ///
"""Autotestes do assistente interativo (wizard.py)."""

from pathlib import Path
import sys
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parent))

from wizard import (  # noqa: E402
    INSTALACOES_PREDEFINIDAS,
    SelOrgaoParser,
    encontrar_binario_ies,
    normalizar_url_sei,
)


class TestWizard(unittest.TestCase):
    def test_instalacoes_predefinidas_contem_campos_obrigatorios(self):
        for chave, info in INSTALACOES_PREDEFINIDAS.items():
            self.assertIn("slug", info)
            self.assertIn("origem", info)
            self.assertIn("usuario", info)
            self.assertIn("senha", info)
            self.assertIn("orgao", info)
            self.assertTrue(info["origem"].startswith("https://"))

    def test_encontrar_binario_ies_retorna_string(self):
        binario = encontrar_binario_ies()
        self.assertIsInstance(binario, str)
        self.assertTrue(len(binario) > 0)

    def test_normalizar_url_sei_variantes(self):
        raiz, origem = normalizar_url_sei("https://sei.sistemas.ro.gov.br")
        self.assertEqual(raiz, "https://sei.sistemas.ro.gov.br")
        self.assertEqual(origem, "https://sei.sistemas.ro.gov.br/sei")

        raiz2, origem2 = normalizar_url_sei(
            "https://sei.sistemas.ro.gov.br/sei/controlador.php?acao=procedimento_trabalhar&id=123"
        )
        self.assertEqual(raiz2, "https://sei.sistemas.ro.gov.br")
        self.assertEqual(origem2, "https://sei.sistemas.ro.gov.br/sei")

        raiz3, origem3 = normalizar_url_sei("https://sei.sistemas.ro.gov.br/sip/login.php?sigla_orgao_sistema=RO")
        self.assertEqual(raiz3, "https://sei.sistemas.ro.gov.br")
        self.assertEqual(origem3, "https://sei.sistemas.ro.gov.br/sei")

    def test_sel_orgao_parser_extrai_opcoes(self):
        html = """
        <html>
            <body>
                <select name="selOrgao" id="selOrgao">
                    <option value="null">Selecione o órgão</option>
                    <option value="1">AGERO</option>
                    <option value="9">PGE</option>
                </select>
            </body>
        </html>
        """
        parser = SelOrgaoParser()
        parser.feed(html)
        self.assertEqual(parser.options, [("1", "AGERO"), ("9", "PGE")])


if __name__ == "__main__":
    unittest.main()
