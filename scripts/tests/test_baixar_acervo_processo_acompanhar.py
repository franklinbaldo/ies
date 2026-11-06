"""Teste sintético do manifesto de downloads do acompanhamento."""
import tempfile
import unittest
from pathlib import Path
from scripts.baixar_acervo_processo_acompanhar import INSTALACOES, ORIGEM, manifestar


class Manifesto(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.raiz = Path(self.temp.name)
        self.sha = "1" * 40
        for instalacao in INSTALACOES:
            pasta = self.raiz / instalacao
            pasta.mkdir()
            (pasta / f"processo-acompanhar-{instalacao}.md").write_text(
                f"---\ntype: Cassette\nsource_zip: {ORIGEM}lotes/lote-abc.zip\narchive_sha1: {self.sha}\n---\n",
                encoding="utf-8",
            )

    def tearDown(self):
        self.temp.cleanup()

    def test_manifesto_deduplica_zip(self):
        self.assertEqual(
            manifestar(self.raiz),
            {"lote-abc.zip": (ORIGEM + "lotes/lote-abc.zip", self.sha)},
        )

    def test_recusa_fonte_fora_do_acervo(self):
        card = self.raiz / "unb" / "processo-acompanhar-unb.md"
        card.write_text(card.read_text().replace(ORIGEM, "https://outro.example/"))
        with self.assertRaisesRegex(ValueError, "origem não autorizada"):
            manifestar(self.raiz)

    def test_recusa_digest_divergente(self):
        card = self.raiz / "unb" / "processo-acompanhar-unb.md"
        card.write_text(card.read_text().replace(self.sha, "2" * 40))
        with self.assertRaisesRegex(ValueError, "origem divergente"):
            manifestar(self.raiz)


if __name__ == "__main__":
    unittest.main()
