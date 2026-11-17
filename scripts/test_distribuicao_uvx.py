"""Verificacoes offline do contrato de distribuicao Rust/uvx (#335)."""

from pathlib import Path
import tomllib
import unittest

RAIZ = Path(__file__).resolve().parents[1]


class DistribuicaoUvxTest(unittest.TestCase):
    def test_pacote_expoe_binario_rust_pelo_maturin(self):
        with (RAIZ / "pyproject.toml").open("rb") as arquivo:
            projeto = tomllib.load(arquivo)
        self.assertEqual(projeto["build-system"]["build-backend"], "maturin")
        self.assertEqual(projeto["tool"]["maturin"]["bindings"], "bin")
        self.assertEqual(
            projeto["tool"]["maturin"]["targets"],
            [{"name": "ies", "kind": "bin"}],
        )
        self.assertNotIn("scripts", projeto["project"])

    def test_nome_pypi_nao_colide_e_versao_acompanha_cargo(self):
        with (RAIZ / "pyproject.toml").open("rb") as arquivo:
            projeto = tomllib.load(arquivo)
        with (RAIZ / "Cargo.toml").open("rb") as arquivo:
            cargo = tomllib.load(arquivo)
        self.assertEqual(projeto["project"]["name"], "ies-sei")
        self.assertEqual(projeto["project"]["version"], cargo["package"]["version"])
        self.assertTrue(projeto["tool"]["maturin"]["locked"])


if __name__ == "__main__":
    unittest.main()
