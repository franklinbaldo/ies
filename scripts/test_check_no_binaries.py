"""Testes sintéticos do gate de artefatos rastreados."""

import pathlib
import unittest

from check_no_binaries import is_binary_artifact


class TestNoBinaries(unittest.TestCase):
    def test_aceita_fontes_e_lock(self):
        self.assertFalse(is_binary_artifact(pathlib.Path("src/main.rs"), b"fn main() {}\n"))
        self.assertFalse(is_binary_artifact(pathlib.Path("Cargo.lock"), b"version = 4\n"))
        self.assertFalse(is_binary_artifact(pathlib.Path("cassettes/go/exemplo.md"), b"---\ntype: Cassette\n"))

    def test_recusa_executavel_elf_sem_extensao(self):
        self.assertTrue(is_binary_artifact(pathlib.Path("ies"), b"\x7fELF\x02\x01"))

    def test_recusa_artefatos_compilados(self):
        for name in ["target/release/ies", "release/app.dll", "dist/ies", "obj/ies.o", "libfoo.so", "mod.wasm", "x.pyd"]:
            with self.subTest(name=name):
                self.assertTrue(is_binary_artifact(pathlib.Path(name), b"whatever"))

    def test_recusa_bytes_binarios_sem_extensao(self):
        self.assertTrue(is_binary_artifact(pathlib.Path("saida"), b"abc\x00def"))
        self.assertTrue(is_binary_artifact(pathlib.Path("saida"), b"\xff\xfe"))

    def test_indice_binario_e_worktree_texto_nao_passa(self):
        import tempfile
        import subprocess
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory)
            subprocess.run(["git", "init", "-q", str(root)], check=True)
            file = root / "ies"
            file.write_bytes(b"\x7fELF\x02\x01")
            subprocess.run(["git", "-C", str(root), "add", "ies"], check=True)
            file.write_text("texto legivel", encoding="utf-8")
            from check_no_binaries import index_violations
            self.assertEqual(index_violations(root), ["ies"])

    def test_indice_texto_e_worktree_binario_nao_bloqueia(self):
        import tempfile
        import subprocess
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory)
            subprocess.run(["git", "init", "-q", str(root)], check=True)
            file = root / "ies"
            file.write_text("texto legivel", encoding="utf-8")
            subprocess.run(["git", "-C", str(root), "add", "ies"], check=True)
            file.write_bytes(b"\x7fELF\x02\x01")
            from check_no_binaries import index_violations
            self.assertEqual(index_violations(root), [])


if __name__ == "__main__":
    unittest.main()
