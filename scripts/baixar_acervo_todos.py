#!/usr/bin/env python3
"""Prepara o acervo publico para replay no GitHub Actions, sem acessar o SEI.

Cada ZIP e cada membro usado por um card tem o hash conferido antes dos
testes Rust. A saida nao mostra corpos, URLs assinadas nem cookies.
"""

import hashlib
import json
import os
import re
import tempfile
from pathlib import Path
from urllib.parse import urlsplit
from urllib.request import urlopen
from zipfile import ZipFile

from inspecionar_cassettes import MAX_MEMBER_BYTES, ler_card

CACHE = Path(os.environ.get("IES_ACERVO_CACHE", ".cache/ies-retornos")).resolve()


def cards_do_acervo():
    cards = []
    for caminho in sorted(Path("cassettes").rglob("*.md")):
        if "source_zip:" not in caminho.read_text(encoding="utf-8"):
            continue
        cards.append((caminho, ler_card(caminho)))
    if not cards:
        raise RuntimeError("Nenhum card com source_zip encontrado")
    return cards


def arquivo_zip(url):
    nome = Path(urlsplit(url).path).name
    if not re.fullmatch(r"[A-Za-z0-9_.-]+\.zip", nome):
        raise ValueError("Nome de ZIP nao autorizado no card")
    return CACHE / nome


def sha1_do_arquivo(caminho):
    sha = hashlib.sha1()
    with caminho.open("rb") as arquivo:
        for pedaco in iter(lambda: arquivo.read(1024 * 1024), b""):
            sha.update(pedaco)
    return sha.hexdigest()


def obter_zip(url, sha_esperado):
    destino = arquivo_zip(url)
    if destino.exists() and sha1_do_arquivo(destino) == sha_esperado:
        return destino

    temporario = None
    try:
        with tempfile.NamedTemporaryFile(dir=CACHE, suffix=".zip", delete=False) as saida:
            temporario = Path(saida.name)
            digest = hashlib.sha1()
            with urlopen(url, timeout=240) as resposta:
                while pedaco := resposta.read(1024 * 1024):
                    saida.write(pedaco)
                    digest.update(pedaco)
        if digest.hexdigest() != sha_esperado:
            raise ValueError(f"SHA-1 divergente no ZIP {destino.name}")
        temporario.replace(destino)
    finally:
        if temporario is not None:
            temporario.unlink(missing_ok=True)
    return destino


def conferir_membro(arquivo, caminho_card, card):
    with ZipFile(arquivo) as zipado:
        membro = zipado.getinfo(card["archive_member"])
        if membro.file_size > MAX_MEMBER_BYTES:
            raise ValueError(f"Membro grande demais: {caminho_card}")
        conteudo = zipado.read(membro)
    sha = hashlib.sha256(conteudo.replace(b"\r\n", b"\n")).hexdigest()
    if sha != card["cassette_sha256_lf"]:
        raise ValueError(f"SHA-256 divergente: {caminho_card}")
    objeto = json.loads(conteudo)
    if objeto.get("schema_version") != 1 or objeto.get("complete") is not True:
        raise ValueError(f"Schema ou completude invalida: {caminho_card}")


def main():
    CACHE.mkdir(parents=True, exist_ok=True)
    cards = cards_do_acervo()
    origens = {}
    for caminho_card, card in cards:
        url = card["source_zip"]
        sha = card["archive_sha1"]
        if url in origens and origens[url] != sha:
            raise ValueError(f"SHA-1 conflitante entre cards: {caminho_card}")
        origens[url] = sha

    arquivos = {url: obter_zip(url, sha) for url, sha in sorted(origens.items())}
    for caminho_card, card in cards:
        conferir_membro(arquivos[card["source_zip"]], caminho_card, card)
    print(f"PASS: {len(cards)} cards e {len(arquivos)} ZIPs conferidos por hash")
    print("Integridade nao comprova paridade: executar replays Rust em seguida.")


if __name__ == "__main__":
    main()
