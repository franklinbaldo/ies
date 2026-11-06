"""Condições de segurança para apagar branches antigas."""

import unittest

from limpar_branches import elegivel


class TestLimparBranches(unittest.TestCase):
    def test_apaga_apenas_integrada_sem_pr(self):
        branch = {"name": "codex/old", "commit": {"sha": "abc"}, "protected": False}
        self.assertTrue(elegivel(branch, "mainsha", set(), 0))
        self.assertFalse(elegivel(branch, "mainsha", set(), 1))
        self.assertFalse(elegivel(branch, "mainsha", {"codex/old"}, 0))

    def test_preserva_main_e_protecao(self):
        for name, protected, sha in [("main", False, "abc"), ("old", True, "abc")]:
            self.assertFalse(elegivel({"name": name, "protected": protected, "commit": {"sha": sha}}, "mainsha", set(), 0))
        self.assertTrue(elegivel({"name": "old", "protected": False, "commit": {"sha": "mainsha"}}, "mainsha", set(), 0))

    def test_lease_rejeita_branch_movida(self):
        import pathlib
        import subprocess
        import tempfile
        from limpar_branches import delete_with_lease

        def git(*cmd, cwd=None):
            return subprocess.run(["git", *cmd], cwd=cwd, check=True,
                                  text=True, capture_output=True).stdout.strip()

        with tempfile.TemporaryDirectory() as d:
            root = pathlib.Path(d)
            origin, work = root / "remote.git", root / "work"
            git("init", "--bare", "-q", str(origin))
            git("init", "-q", str(work))
            git("config", "user.email", "tests@example.invalid", cwd=work)
            git("config", "user.name", "Fixture", cwd=work)
            file = work / "a.txt"
            file.write_text("primeiro", encoding="utf-8")
            git("add", "a.txt", cwd=work)
            git("commit", "-qm", "primeiro", cwd=work)
            sha_antigo = git("rev-parse", "HEAD", cwd=work)
            git("remote", "add", "origin", str(origin), cwd=work)
            git("push", "-q", "origin", "HEAD:refs/heads/antiga", cwd=work)
            file.write_text("segundo", encoding="utf-8")
            git("commit", "-qam", "segundo", cwd=work)
            git("push", "-q", "origin", "HEAD:refs/heads/antiga", cwd=work)

            with self.assertRaises(subprocess.CalledProcessError):
                delete_with_lease("antiga", sha_antigo, cwd=work)
            self.assertTrue(git("ls-remote", "--heads", "origin", "antiga", cwd=work))

            sha_novo = git("rev-parse", "HEAD", cwd=work)
            delete_with_lease("antiga", sha_novo, cwd=work)
            self.assertFalse(git("ls-remote", "--heads", "origin", "antiga", cwd=work))


if __name__ == "__main__":
    unittest.main()
