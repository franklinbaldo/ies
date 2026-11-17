"""Testes totalmente sintéticos da obtenção de ZIPs para ciência (#271)."""
import hashlib
import io
import tempfile
import unittest
from pathlib import Path
from unittest.mock import Mock

from scripts.baixar_acervo_ciencia import INSTALACOES, ORIGEM, manifestar, preparar

DADOS = b"ZIP sintetico sem dados do SEI"
SHA = hashlib.sha1(DADOS).hexdigest()


class TestBaixarAcervoCiencia(unittest.TestCase):
    def setUp(self):
        temp = tempfile.TemporaryDirectory()
        self.addCleanup(temp.cleanup)
        self.raiz = Path(temp.name) / "cassettes"
        self.cache = Path(temp.name) / "cache"
        for instalacao in INSTALACOES:
            (self.raiz / instalacao).mkdir(parents=True)
            for alvo in ("processo", "documento"):
                self.card(instalacao, alvo)

    def card(self, instalacao, alvo, url=None, sha=SHA):
        url = url or f"{ORIGEM}todos-sei-cassettes-{instalacao}.zip"
        path = self.raiz / instalacao / f"{alvo}-ciente-{instalacao}.md"
        path.write_text(
            f"---\ntype: Cassette\nsource_zip: {url}\n"
            f"archive_sha1: {sha}\n---\n# fixture\n", encoding="utf-8"
        )

    def test_manifesto_dos_14_cards_e_deduplicacao(self):
        manifesto = manifestar(self.raiz)
        self.assertEqual(len(manifesto), 7)
        self.assertEqual(manifesto["todos-sei-cassettes-unb.zip"][1], SHA)

    def test_lote_publico_permitido(self):
        url = ORIGEM + "lotes/lote-c7a4af8d9d70ddc3.zip"
        self.card("go", "documento", url=url)
        manifesto = manifestar(self.raiz)
        self.assertIn("lote-c7a4af8d9d70ddc3.zip", manifesto)

    def test_recusa_card_faltando_ou_origem_externa(self):
        (self.raiz / "unb" / "documento-ciente-unb.md").unlink()
        with self.assertRaises(ValueError):
            manifestar(self.raiz)
        self.card("unb", "documento", url="https://exemplo.invalid/zip")
        with self.assertRaises(ValueError):
            manifestar(self.raiz)

    def test_recusa_caminho_estranho_ou_hash_diferente(self):
        self.card("go", "documento", url=ORIGEM + "lotes/../segredo.zip")
        with self.assertRaises(ValueError):
            manifestar(self.raiz)
        self.card("go", "documento")
        self.card("go", "documento", sha="0" * 40)
        with self.assertRaises(ValueError):
            manifestar(self.raiz)

    def test_download_uma_vez_e_cache_revalidado(self):
        abrir = Mock(side_effect=lambda url, timeout: io.BytesIO(DADOS))
        preparar(self.raiz, self.cache, abrir)
        preparar(self.raiz, self.cache, abrir)
        self.assertEqual(abrir.call_count, 7)

    def test_download_adulterado_nao_publica_e_nao_repete(self):
        abrir = Mock(side_effect=lambda url, timeout: io.BytesIO(b"invalidos"))
        with self.assertRaises(ValueError):
            preparar(self.raiz, self.cache, abrir)
        self.assertEqual(abrir.call_count, 1)
        self.assertFalse(list(self.cache.iterdir()))

    def test_cache_adulterado_falha_antes_da_rede(self):
        self.cache.mkdir()
        (self.cache / "todos-sei-cassettes-go.zip").write_bytes(b"invalidos")
        abrir = Mock()
        with self.assertRaises(ValueError):
            preparar(self.raiz, self.cache, abrir)
        abrir.assert_not_called()


if __name__ == "__main__":
    unittest.main()
