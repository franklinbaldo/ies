#!/usr/bin/env python3
"""ZIPs públicos do acompanhamento, com hash obrigatório; nunca acessa SEI."""
from __future__ import annotations

import hashlib
import os
from pathlib import Path
import tempfile
from urllib.request import urlopen

ORIGEM = "https://archive.org/download/todos-sei-cassettes-2026/"
INSTALACOES = ("unb", "ufrj", "go", "pen1", "pen2", "pen3", "pen4")
LIMITE_BYTES = 160_000_000


def manifestar(raiz: Path) -> dict[str, tuple[str, str]]:
    manifesto: dict[str, tuple[str, str]] = {}
    encontradas: set[str] = set()
    for instalacao in INSTALACOES:
        for card in sorted((raiz / instalacao).glob("processo-acompanhar-*.md")):
            encontradas.add(instalacao)
            texto = card.read_text(encoding="utf-8")
            if not texto.startswith("---\n") or "\n---\n" not in texto:
                raise ValueError(f"frontmatter ausente: {card}")
            campos: dict[str, str] = {}
            for linha in texto.split("\n---\n", 1)[0].splitlines()[1:]:
                if ": " not in linha:
                    continue
                chave, valor = linha.split(": ", 1)
                if chave in campos:
                    raise ValueError(f"campo repetido: {card}")
                campos[chave] = valor.strip()
            url = campos.get("source_zip", "")
            if not url.startswith(ORIGEM):
                raise ValueError(f"origem não autorizada: {card}")
            caminho = url[len(ORIGEM):]
            if (".." in caminho or "\\" in caminho or not caminho.endswith(".zip")
                    or not (caminho.startswith("lotes/lote-")
                            or caminho.startswith("todos-sei-cassettes-"))):
                raise ValueError(f"ZIP não permitido: {card}")
            nome = caminho.rsplit("/", 1)[-1]
            if not nome or any(c not in "abcdefghijklmnopqrstuvwxyz0123456789-." for c in nome):
                raise ValueError(f"nome inseguro: {card}")
            sha = campos.get("archive_sha1", "")
            if len(sha) != 40 or any(c not in "0123456789abcdef" for c in sha):
                raise ValueError(f"hash inválido: {card}")
            if nome in manifesto and manifesto[nome] != (url, sha):
                raise ValueError(f"origem divergente: {card}")
            manifesto[nome] = (url, sha)
    if encontradas != set(INSTALACOES):
        raise ValueError("cassettes ausentes de alguma instalação")
    return manifesto


def sha1(caminho: Path) -> str:
    digest = hashlib.sha1()
    with caminho.open("rb") as entrada:
        for bloco in iter(lambda: entrada.read(1024 * 1024), b""):
            digest.update(bloco)
    return digest.hexdigest()


def preparar(raiz: Path, cache: Path, abrir=urlopen) -> None:
    manifesto = manifestar(raiz)
    cache.mkdir(parents=True, exist_ok=True)
    for nome, (url, esperado) in sorted(manifesto.items()):
        destino = cache / nome
        if destino.exists():
            if not destino.is_file() or sha1(destino) != esperado:
                raise ValueError(f"cache divergente: {nome}")
            print(f"CACHE verificado: {nome}")
            continue
        temporario: Path | None = None
        try:
            with tempfile.NamedTemporaryFile(dir=cache, prefix=".acervo-", delete=False) as f:
                temporario = Path(f.name)
                total = 0
                with abrir(url, timeout=120) as resposta:
                    for bloco in iter(lambda: resposta.read(1024 * 1024), b""):
                        total += len(bloco)
                        if total > LIMITE_BYTES:
                            raise ValueError(f"ZIP acima de 160 MB: {nome}")
                        f.write(bloco)
            if sha1(temporario) != esperado:
                raise ValueError(f"SHA-1 divergente: {nome}")
            os.replace(temporario, destino)
            temporario = None
            print(f"ZIP confirmado: {nome}")
        finally:
            if temporario is not None:
                temporario.unlink(missing_ok=True)


if __name__ == "__main__":
    raiz = Path(__file__).resolve().parent.parent / "cassettes"
    cache = Path(os.environ.get("IES_ACERVO_CACHE", ".cache/ies"))
    preparar(raiz, cache)
