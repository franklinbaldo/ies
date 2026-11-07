"""Testes offline do incremento SemVer da distribuicao IES (#346)."""

from pathlib import Path
import subprocess
import sys
import tempfile
import tomllib
import unittest

SCRIPT = Path(__file__).resolve().parent / "versao_semver.py"


class IncrementoSemVerTest(unittest.TestCase):
    def setUp(self):
        self.temporario = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporario.cleanup)
        self.raiz = Path(self.temporario.name)
        (self.raiz / "Cargo.toml").write_text(
            '[package]\nname = "ies"\nversion = "0.1.0"\n'
            '[dependencies]\nclap = { version = "4.5" }\n',
            encoding="utf-8",
        )
        (self.raiz / "pyproject.toml").write_text(
            '[project]\nname = "ies-sei"\nversion = "0.1.0"\n'
            '[tool.maturin]\nbindings = "bin"\n',
            encoding="utf-8",
        )
        (self.raiz / "Cargo.lock").write_text(
            'version = 4\n\n[[package]]\nname = "ies"\nversion = "0.1.0"\n'
            '\n[[package]]\nname = "ies-core"\nversion = "0.1.0"\n',
            encoding="utf-8",
        )

    def executar(self, tipo, *opcoes):
        return subprocess.run(
            [sys.executable, "-I", str(SCRIPT), tipo, str(self.raiz), *opcoes],
            capture_output=True, text=True, check=False,
        )

    def versoes(self):
        versoes = []
        for arquivo, chave in (
            ("Cargo.toml", "package"),
            ("pyproject.toml", "project"),
            ("Cargo.lock", "packages"),
        ):
            with (self.raiz / arquivo).open("rb") as entrada:
                dados = tomllib.load(entrada)
            if chave == "packages":
                versoes.append(next(p["version"] for p in dados["package"] if p["name"] == "ies"))
            else:
                versoes.append(dados[chave]["version"])
        return versoes

    def test_dry_run_nao_altera_arquivos(self):
        resultado = self.executar("patch")
        self.assertEqual(resultado.returncode, 0, resultado.stderr)
        self.assertIn("0.1.1", resultado.stdout)
        self.assertEqual(self.versoes(), ["0.1.0"] * 3)

    def test_patch_aplica_aos_tres_manifestos(self):
        resultado = self.executar("patch", "--aplicar")
        self.assertEqual(resultado.returncode, 0, resultado.stderr)
        self.assertEqual(self.versoes(), ["0.1.1"] * 3)
        self.assertIn('version = "4.5"', (self.raiz / "Cargo.toml").read_text())

    def test_minor_reinicia_patch(self):
        resultado = self.executar("minor", "--aplicar")
        self.assertEqual(resultado.returncode, 0, resultado.stderr)
        self.assertEqual(self.versoes(), ["0.2.0"] * 3)

    def test_major_reinicia_minor_e_patch(self):
        resultado = self.executar("major", "--aplicar")
        self.assertEqual(resultado.returncode, 0, resultado.stderr)
        self.assertEqual(self.versoes(), ["1.0.0"] * 3)

    def test_recusa_versao_desincronizada_sem_tocar_em_nada(self):
        arquivo = self.raiz / "pyproject.toml"
        arquivo.write_text(arquivo.read_text().replace("0.1.0", "0.2.0"))
        antes = {p.name: p.read_bytes() for p in self.raiz.iterdir()}
        resultado = self.executar("patch", "--aplicar")
        self.assertNotEqual(resultado.returncode, 0)
        self.assertEqual(antes, {p.name: p.read_bytes() for p in self.raiz.iterdir()})

    def test_recusa_lock_desincronizado(self):
        arquivo = self.raiz / "Cargo.lock"
        arquivo.write_text(arquivo.read_text().replace('name = "ies"\nversion = "0.1.0"',
                                                             'name = "ies"\nversion = "0.0.9"'))
        resultado = self.executar("minor", "--aplicar")
        self.assertNotEqual(resultado.returncode, 0)
        self.assertEqual(self.versoes(), ["0.1.0", "0.1.0", "0.0.9"])

    def test_recusa_prerelease_ate_suporte_explicito(self):
        arquivo = self.raiz / "Cargo.toml"
        arquivo.write_text(arquivo.read_text().replace('version = "0.1.0"',
                                                             'version = "0.1.0-rc.1"'))
        resultado = self.executar("patch", "--aplicar")
        self.assertNotEqual(resultado.returncode, 0)


if __name__ == "__main__":
    unittest.main()
