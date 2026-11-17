"""Checks de bump SemVer no CI canonico: somente a serie 0.x.y (#346)."""

import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

SCRIPT = Path(__file__).resolve().parent / "check_bump_ci.py"


class CheckBumpCiTest(unittest.TestCase):
    def setUp(self):
        self.temporario = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporario.cleanup)
        self.raiz = Path(self.temporario.name)
        self.gravar("0.1.0")

    def gravar(self, versao, py=None, lock=None):
        (self.raiz / "Cargo.toml").write_text(
            f'[package]\nname = "ies"\nversion = "{versao}"\n', encoding="utf-8"
        )
        (self.raiz / "pyproject.toml").write_text(
            f'[project]\nname = "ies-sei"\nversion = "{py or versao}"\n',
            encoding="utf-8",
        )
        (self.raiz / "Cargo.lock").write_text(
            f'version = 4\n\n[[package]]\nname = "ies"\nversion = "{lock or versao}"\n',
            encoding="utf-8",
        )

    def git(self, *args):
        return subprocess.run(
            ["git", "-C", str(self.raiz), *args], capture_output=True,
            text=True, check=True,
        )

    def preparar_git(self, anterior, atual):
        self.gravar(anterior)
        self.git("init", "-q")
        self.git("config", "user.name", "CI Synthetic")
        self.git("config", "user.email", "ci@example.invalid")
        self.git("add", ".")
        self.git("commit", "-qm", "base")
        self.gravar(atual)
        self.git("add", ".")
        self.git("commit", "--allow-empty", "-qm", "candidate")

    def verificar(self, evento=None, ref=None):
        env = os.environ.copy()
        env["GITHUB_EVENT_NAME"] = evento or ""
        env["GITHUB_REF"] = ref or ""
        return subprocess.run(
            [sys.executable, "-I", str(SCRIPT), str(self.raiz)],
            capture_output=True, text=True, check=False, env=env,
        )

    def test_consistente_0_x_sem_bump(self):
        resultado = self.verificar()
        self.assertEqual(resultado.returncode, 0, resultado.stderr)

    def test_divergencia_dos_manifestos_bloqueia(self):
        self.gravar("0.1.0", py="0.2.0")
        resultado = self.verificar()
        self.assertNotEqual(resultado.returncode, 0)

    def test_divergencia_do_lock_bloqueia(self):
        self.gravar("0.1.0", lock="0.1.1")
        self.assertNotEqual(self.verificar().returncode, 0)

    def test_versao_major_1_bloqueia(self):
        self.gravar("1.0.0")
        self.assertNotEqual(self.verificar().returncode, 0)

    def test_prerelease_nao_aceita(self):
        self.gravar("0.2.0-rc.1")
        self.assertNotEqual(self.verificar().returncode, 0)

    def test_patch_incrementado_na_main(self):
        self.preparar_git("0.1.0", "0.1.1")
        resultado = self.verificar(evento="push", ref="refs/heads/main")
        self.assertEqual(resultado.returncode, 0, resultado.stderr)

    def test_minor_incrementado_na_main(self):
        self.preparar_git("0.1.9", "0.2.0")
        self.assertEqual(
            self.verificar(evento="push", ref="refs/heads/main").returncode, 0
        )

    def test_regressao_de_versao_na_main_bloqueia(self):
        self.preparar_git("0.2.0", "0.1.9")
        self.assertNotEqual(
            self.verificar(evento="push", ref="refs/heads/main").returncode, 0
        )

    def test_pr_sem_bump_nao_quebra_rotina(self):
        self.preparar_git("0.1.0", "0.1.0")
        self.assertEqual(
            self.verificar(evento="pull_request", ref="refs/pull/123/merge").returncode, 0
        )


if __name__ == "__main__":
    unittest.main()
