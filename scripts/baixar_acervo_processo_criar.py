#!/usr/bin/env python3
"""Obtém ZIPs públicos para o replay obrigatório de processo-criar (#245).

Não acessa o SEI. A lista de origens/hashes vem exclusivamente dos cards
imutáveis no Git; um download por ZIP, verificado antes da publicação.
"""
from __future__ import annotations

import hashlib
import os
from pathlib import Path
import tempfile
from urllib.request import urlopen

ORIGEM = "https://archive.org/download/todos-sei-cassettes-2026/"
INSTALACOES = ("unb", "ufrj", "go", "pen1", "pen2", "pen3", "pen4")
LIMITE_BYTES = 512 * 1024 * 1024
MINIMO_CARDS = 15  # Cobertura já demonstrada pelo replay #245 em 05/11/2026


def manifestar(raiz: Path) -> dict[str, str]:
    """Lê os hashes declarados e rejeita fontes, nomes ou hashes divergentes."""
    manifesto: dict[str, str] = {}
    cards = 0
    for instalacao in INSTALACOES:
        pasta = raiz / instalacao
        for card in sorted(pasta.glob("processo-criar-*.md")):
            cards += 1
            texto = card.read_text(encoding="utf-8")
            if not texto.startswith("---\n"):
                raise ValueError(f"frontmatter ausente: {card.name}")
            cabecalho = texto.split("---", 2)[1]
            campos: dict[str, str] = {}
            for linha in cabecalho.splitlines():
                if ": " in linha:
                    chave, valor = linha.split(": ", 1)
                    if chave in campos:
                        raise ValueError(f"campo repetido: {card.name}")
                    campos[chave] = valor.strip()
            nome = f"todos-sei-cassettes-{instalacao}.zip"
            if campos.get("source_zip") != ORIGEM + nome:
                raise ValueError(f"origem não autorizada: {card.name}")
            sha = campos.get("archive_sha1", "")
            if len(sha) != 40 or any(c not in "0123456789abcdef" for c in sha):
                raise ValueError(f"SHA-1 inválido: {card.name}")
            if nome in manifesto and manifesto[nome] != sha:
                raise ValueError(f"SHA-1 conflitante: {card.name}")
            manifesto[nome] = sha
    if cards < MINIMO_CARDS or len(manifesto) != len(INSTALACOES):
        raise ValueError("acervo incompleto: faltam cards de uma instalação")
    return manifesto


def _sha1(caminho: Path) -> str:
    digest = hashlib.sha1()
    with caminho.open("rb") as entrada:
        for bloco in iter(lambda: entrada.read(1024 * 1024), b""):
            digest.update(bloco)
    return digest.hexdigest()


def preparar(raiz: Path, cache: Path, abrir=urlopen) -> None:
    """Baixa no máximo uma vez por ZIP, nunca usa byte sem hash válido."""
    manifesto = manifestar(raiz)
    cache.mkdir(parents=True, exist_ok=True)
    for nome, sha in sorted(manifesto.items()):
        destino = cache / nome
        if destino.exists():
            if not destino.is_file() or _sha1(destino) != sha:
                raise ValueError(f"cache divergente: {nome}")
            print(f"CACHE verificado: {nome}")
            continue
        temporario: Path | None = None
        try:
            with tempfile.NamedTemporaryFile(dir=cache, prefix=".acervo-", delete=False) as arquivo:
                temporario = Path(arquivo.name)
                total = 0
                with abrir(ORIGEM + nome, timeout=120) as resposta:
                    for bloco in iter(lambda: resposta.read(1024 * 1024), b""):
                        total += len(bloco)
                        if total > LIMITE_BYTES:
                            raise ValueError(f"ZIP excedeu limite: {nome}")
                        arquivo.write(bloco)
            if _sha1(temporario) != sha:
                raise ValueError(f"SHA-1 divergente: {nome}")
            os.replace(temporario, destino)
            temporario = None
            print(f"ZIP verificado: {nome}")
        finally:
            if temporario is not None:
                temporario.unlink(missing_ok=True)


def main() -> None:
    raiz = Path(__file__).resolve().parent.parent / "cassettes"
    cache = Path(os.environ.get("IES_ACERVO_CACHE", ".cache/ies"))
    preparar(raiz, cache)


if __name__ == "__main__":
    main()
