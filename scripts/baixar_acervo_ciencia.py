#!/usr/bin/env python3
"""Obtém, sem acessar o SEI, os ZIPs públicos da ciência (#271).

Usa exclusivamente a origem e o SHA-1 declarados nos 14 cards da
cassette. Nunca reutiliza cache inválido, nem repete download fracassado.
"""
from __future__ import annotations

import hashlib
import os
from pathlib import Path
import tempfile
from urllib.request import urlopen

ORIGEM = "https://archive.org/download/todos-sei-cassettes-2026/"
INSTALACOES = ("unb", "ufrj", "go", "pen1", "pen2", "pen3", "pen4")
ALVOS = ("processo-ciente-", "documento-ciente-")
LIMITE_BYTES = 512 * 1024 * 1024


def _metadados(card: Path) -> dict[str, str]:
    texto = card.read_text(encoding="utf-8")
    if not texto.startswith("---\n"):
        raise ValueError(f"frontmatter ausente: {card.name}")
    fim = texto.find("\n---\n", 4)
    if fim == -1:
        raise ValueError(f"frontmatter incompleto: {card.name}")
    campos: dict[str, str] = {}
    for linha in texto[4:fim].splitlines():
        if ": " not in linha:
            continue
        chave, valor = linha.split(": ", 1)
        if chave in campos:
            raise ValueError(f"campo repetido: {card.name}: {chave}")
        campos[chave] = valor.strip()
    return campos


def manifestar(raiz: Path) -> dict[str, tuple[str, str]]:
    """Mapeia nome local a (URL permitida, SHA-1), validando 14 cards."""
    manifesto: dict[str, tuple[str, str]] = {}
    for instalacao in INSTALACOES:
        pasta = raiz / instalacao
        for alvo in ALVOS:
            cards = sorted(pasta.glob(f"{alvo}*.md"))
            if len(cards) != 1:
                raise ValueError(f"esperado um card {alvo} em {instalacao}")
            card = cards[0]
            campos = _metadados(card)
            url = campos.get("source_zip", "")
            if not url.startswith(ORIGEM):
                raise ValueError(f"origem não permitida: {card.name}")
            caminho = url[len(ORIGEM):]
            partes = caminho.split("/")
            permitido = (
                len(partes) == 1
                and partes[0] == f"todos-sei-cassettes-{instalacao}.zip"
            ) or (
                len(partes) == 2
                and partes[0] == "lotes"
                and partes[1].startswith("lote-")
                and partes[1].endswith(".zip")
                and all(c in "0123456789abcdef" for c in partes[1][5:-4])
                and len(partes[1][5:-4]) == 16
            )
            if not permitido:
                raise ValueError(f"caminho de ZIP não permitido: {card.name}")
            nome = partes[-1]
            sha = campos.get("archive_sha1", "")
            if len(sha) != 40 or any(c not in "0123456789abcdef" for c in sha):
                raise ValueError(f"SHA-1 inválido: {card.name}")
            origem_verificada = (url, sha)
            if nome in manifesto and manifesto[nome] != origem_verificada:
                raise ValueError(f"ZIP com fontes ou hashes conflitantes: {nome}")
            manifesto[nome] = origem_verificada
    return manifesto


def _sha1(caminho: Path) -> str:
    digest = hashlib.sha1()
    with caminho.open("rb") as entrada:
        for bloco in iter(lambda: entrada.read(1024 * 1024), b""):
            digest.update(bloco)
    return digest.hexdigest()


def preparar(raiz: Path, cache: Path, abrir=urlopen) -> None:
    """Um download por ZIP ausente; valida antes de publicar no cache."""
    manifesto = manifestar(raiz)
    cache.mkdir(parents=True, exist_ok=True)
    for nome, (url, sha) in sorted(manifesto.items()):
        destino = cache / nome
        if destino.exists():
            if not destino.is_file() or _sha1(destino) != sha:
                raise ValueError(f"cache inválido: {nome}")
            print(f"CACHE verificado: {nome}")
            continue
        temporario: Path | None = None
        try:
            with tempfile.NamedTemporaryFile(
                dir=cache, prefix=".ciencia-", delete=False
            ) as arquivo:
                temporario = Path(arquivo.name)
                total = 0
                with abrir(url, timeout=180) as resposta:
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
