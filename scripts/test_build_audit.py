"""Autotestes offline do relatório de tempos de build."""

import unittest
from build_audit import diferenca_percentual


class TestDiferencaPercentual(unittest.TestCase):
    def test_reducao_e_aumento(self):
        self.assertEqual(diferenca_percentual(10, 7), -30.0)
        self.assertEqual(diferenca_percentual(10, 12), 20.0)

    def test_base_zero_nao_produz_divisao(self):
        self.assertIsNone(diferenca_percentual(0, 10))


if __name__ == "__main__":
    unittest.main()
