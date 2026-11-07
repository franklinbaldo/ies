"""Contrato de versoes e tags da publicacao GitHub Releases (#346)."""

from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

RAIZ = Path(__file__).resolve().parents[1]
VERIFICADOR = RAIZ / "scripts" / "verificar_release_uvx.py"


class ReleaseUvxTest(unittest.TestCase):
    def validar(self, tag, cargo="0.1.0", python="0.1.0", lock="0.1.0"):
        with tempfile.TemporaryDirectory() as pasta:
            raiz = Path(pasta)
            (raiz / "Cargo.toml").write_text(
                f'[package]\nname = "ies"\nversion = "{cargo}"\n',
                encoding="utf-8",
            )
            (raiz / "pyproject.toml").write_text(
                f'[project]\nname = "ies-sei"\nversion = "{python}"\n',
                encoding="utf-8",
            )
            (raiz / "Cargo.lock").write_text(
                f\'version = 4\
\
[[package]]\
name = "ies"\
version = "{lock}"\
\',
                encoding="utf-8",
            )
            return subprocess.run(
                [sys.executable, "-I", str(VERIFICADOR), tag, str(raiz)],
                capture_output=True,
                text=True,
                check=False,
            )

    def test_tag_igual_versoes_rust_e_python(self):
        resultado = self.validar("v0.1.0")
        self.assertEqual(resultado.returncode, 0, resultado.stderr)

    def test_rejeita_divergencia_versoes(self):
        resultado = self.validar("v0.1.0", python="0.2.0")
        self.assertNotEqual(resultado.returncode, 0)

    def test_rejeita_tag_nao_semver(self):
        for tag in ("latest", "v0.1", "v0.1.0-extra", "v0.1.0/../x"):
            with self.subTest(tag=tag):
                resultado = self.validar(tag)
                self.assertNotEqual(resultado.returncode, 0)

    def test_rejeita_cargo_lock_divergente(self):\n        resultado = self.validar("v0.1.0", lock="0.0.9")\n        self.assertNotEqual(resultado.returncode, 0)\n\n    def test_rejeita_tag_versao_diferente(self):
        resultado = self.validar("v0.2.0")
        self.assertNotEqual(resultado.returncode, 0)


if __name__ == "__main__":
    unittest.main()
