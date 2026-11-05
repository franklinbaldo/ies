import io
import os
import tempfile
import unittest
from contextlib import redirect_stdout
from pathlib import Path
from unittest import mock

import wip_ignores as w

PASSA = '#[test]\nfn a() {}\n'
WIP = '#[test]\n#[ignore = "wip #12: falta o POST"]\nfn b() {}\n'


class TestTestes(unittest.TestCase):
    def test_le_motivos(self):
        fonte = PASSA + WIP + '#[ignore]\n#[test]\nfn c() {}\n#[cfg(test)]\nmod m {}\nfn ajuda() {}\n'
        self.assertEqual(w.testes(fonte), {'a': None, 'b': 'wip #12: falta o POST', 'c': ''})


class TestVerificar(unittest.TestCase):
    def rodar(self, atual, antigo):
        with tempfile.TemporaryDirectory() as d:
            raiz = Path(d)
            (raiz / 'tests').mkdir()
            (raiz / 'tests/x.rs').write_text(atual, encoding='utf-8')
            saida = io.StringIO()
            with redirect_stdout(saida), mock.patch.dict(os.environ, {'CI': ''}):
                codigo = w.verificar(raiz, ler_base=lambda _: antigo)
            return codigo, saida.getvalue()

    def test_wip_novo_passa_e_conta(self):
        codigo, saida = self.rodar(PASSA + WIP, PASSA)
        self.assertEqual(codigo, 0)
        self.assertIn('PENDENTE #12: 1 teste(s)', saida)

    def test_teste_antigo_ignorado_falha(self):
        atual = '#[test]\n#[ignore = "wip #12: quebrou"]\nfn a() {}\n'
        codigo, saida = self.rodar(atual, PASSA)
        self.assertEqual(codigo, 1)
        self.assertIn('já passava na base', saida)

    def test_sem_motivo_falha(self):
        codigo, saida = self.rodar('#[test]\n#[ignore]\nfn z() {}\n', None)
        self.assertEqual(codigo, 1)
        self.assertIn('sem motivo', saida)

    def test_wip_sem_issue_falha(self):
        codigo, _ = self.rodar('#[test]\n#[ignore = "wip: depois"]\nfn z() {}\n', None)
        self.assertEqual(codigo, 1)

    def test_ignore_legitimo_ja_existente_passa(self):
        acervo = '#[test]\n#[ignore = "lê o acervo local"]\nfn r() {}\n'
        codigo, _ = self.rodar(acervo, acervo)
        self.assertEqual(codigo, 0)


if __name__ == '__main__':
    unittest.main()
