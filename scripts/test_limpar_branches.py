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


if __name__ == "__main__":
    unittest.main()
