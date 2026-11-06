"""Regras de obtenção do acervo de teste #245 (nenhuma rede real)."""
import hashlib
import io
import tempfile
import unittest
from pathlib import Path
from unittest.mock import Mock

from scripts.baixar_acervo_processo_criar import INSTALACOES, manifestar, preparar


ORIGEM = "https://archive.org/download/todos-sei-cassettes-2026/"
CONTEUDO = b"ZIP sintetico: nao contem dados de SEI"
SHA1 = hashlib.sha1(CONTEUDO).hexdigest()


class TestBaixarAcervoProcessoCriar(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.raiz = Path(self.temp.name) / "cassettes"
        self.cache = Path(self.temp.name) / "cache"
        for inst in INSTALACOES:
            (self.raiz / inst).mkdir(parents=True)
            self.card(instalacao=inst, nome="base")
            self.card(instalacao=inst, nome="segundo")
        self.card(instalacao="unb", nome="terceiro")

    def card(self, instalacao="unb", nome="um", origem=None, sha=SHA1):
        url = origem or f"{ORIGEM}todos-sei-cassettes-{instalacao}.zip"
        (self.raiz / instalacao / f"processo-criar-{nome}.md").write_text(
            "---\ntype: Cassette\n" + f"source_zip: {url}\narchive_sha1: {sha}\n" +
            "---\n# fixture sintética\n", encoding="utf-8"
        )

    def test_manifesto_usa_cards_e_consolida_zip_repetido(self):
        self.card(nome="a")
        self.card(nome="b")
        self.card(instalacao="go")
        manifesto = manifestar(self.raiz)
        self.assertEqual(set(manifesto), {f"todos-sei-cassettes-{inst}.zip" for inst in INSTALACOES})
        self.assertEqual(manifesto["todos-sei-cassettes-unb.zip"], SHA1)

    def test_manifesto_nao_pode_perder_card_da_cobertura_original(self):
        (self.raiz / "unb" / "processo-criar-terceiro.md").unlink()
        with self.assertRaises(ValueError):
            manifestar(self.raiz)

    def test_manifesto_recusa_origem_inesperada_ou_hash_divergente(self):
        self.card(origem="https://exemplo.invalid/segredo.zip")
        with self.assertRaises(ValueError):
            manifestar(self.raiz)
        (self.raiz / "unb" / "processo-criar-um.md").unlink()
        self.card(nome="um")
        self.card(nome="dois", sha="0" * 40)
        with self.assertRaises(ValueError):
            manifestar(self.raiz)

    def test_download_uma_vez_com_sha_e_cache_valido(self):
        self.card()
        abrir = Mock(side_effect=lambda url, timeout: io.BytesIO(CONTEUDO))
        preparar(self.raiz, self.cache, abrir)
        self.assertEqual((self.cache / "todos-sei-cassettes-unb.zip").read_bytes(), CONTEUDO)
        preparar(self.raiz, self.cache, abrir)
        self.assertEqual(abrir.call_count, len(INSTALACOES))

    def test_download_incorreto_nao_publica_zip_nem_tem_retry(self):
        self.card()
        abrir = Mock(side_effect=lambda url, timeout: io.BytesIO(b"conteudo adulterado"))
        with self.assertRaises(ValueError):
            preparar(self.raiz, self.cache, abrir)
        self.assertEqual(abrir.call_count, 1)
        self.assertFalse((self.cache / "todos-sei-cassettes-go.zip").exists())
        self.assertEqual(list(self.cache.iterdir()), [])

    def test_cache_invalido_falha_fechado_sem_baixar(self):
        self.card()
        self.cache.mkdir()
        (self.cache / "todos-sei-cassettes-go.zip").write_bytes(b"arquivo adulterado")
        abrir = Mock()
        with self.assertRaises(ValueError):
            preparar(self.raiz, self.cache, abrir)
        abrir.assert_not_called()


if __name__ == "__main__":
    unittest.main()
