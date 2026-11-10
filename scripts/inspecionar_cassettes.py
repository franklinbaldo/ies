#!/usr/bin/env python3
"""Inspecao somente leitura de cassettes publicas, com hashes e logs saneados."""

import collections
import hashlib
import json
import os
from pathlib import Path
import re
import sys
import tempfile
from urllib.request import urlopen
from urllib.parse import parse_qs, urlsplit
from zipfile import ZipFile

CARDS_ROOT = Path("cassettes").resolve()
PREFIX = "https://archive.org/download/todos-sei-cassettes-2026/"
PEDIDO = Path(".github/cassettes-a-inspecionar.txt")
MAX_MEMBER_BYTES = 100 * 1024 * 1024


def ler_card(caminho):
    card = Path(caminho).resolve()
    if card.suffix != ".md" or CARDS_ROOT not in card.parents:
        raise ValueError("Card fora de cassettes/ ou sem extensao .md")
    texto = card.read_text(encoding="utf-8")
    if not texto.startswith("---\n"):
        raise ValueError(f"Frontmatter ausente: {card.name}")
    campos = {}
    for linha in texto.split("---", 2)[1].splitlines():
        if ": " in linha:
            chave, valor = linha.split(": ", 1)
            campos[chave] = valor.strip().strip("'\"")
    necessarios = (
        "source_zip", "archive_member", "archive_sha1",
        "cassette_sha256_lf", "interactions", "installation",
    )
    if any(chave not in campos for chave in necessarios):
        raise ValueError(f"Metadados incompletos: {card.name}")
    if not campos["source_zip"].startswith(PREFIX):
        raise ValueError(f"Fonte publica nao autorizada: {card.name}")
    if not re.fullmatch(r"[0-9a-f]{40}", campos["archive_sha1"]):
        raise ValueError(f"SHA-1 invalido: {card.name}")
    if not re.fullmatch(r"[0-9a-f]{64}", campos["cassette_sha256_lf"]):
        raise ValueError(f"SHA-256 invalido: {card.name}")
    return campos


def detalhes_requisicao(req):
    if not isinstance(req, dict):
        return ("?", None, "<nao-dicionario>")
    metodo = req.get("method") or req.get("metodo") or "?"
    acao = None
    def procurar(obj, nivel=0):
        nonlocal acao
        if acao or nivel > 4:
            return
        if isinstance(obj, dict):
            for chave, valor in obj.items():
                if chave in ("acao", "action", "operation") and isinstance(valor, str):
                    if re.fullmatch(r"[a-zA-Z0-9_]+", valor):
                        acao = valor
                        return
                if isinstance(valor, str) and chave in (
                    "url", "path", "uri", "target", "request_url", "href", "full",
                    "full_url", "original", "raw",
                ):
                    match = re.search(r"(?:[?&]|^)acao=([a-zA-Z0-9_]+)", valor)
                    if match:
                        acao = match.group(1)
                        return
                if isinstance(valor, (dict, list)):
                    procurar(valor, nivel + 1)
        elif isinstance(obj, list):
            for item in obj:
                procurar(item, nivel + 1)
    procurar(req)
    estrutura = ",".join(sorted(req.keys()))
    return (str(metodo).upper(), acao, estrutura)


def verificar(caminho, arquivos):
    card = ler_card(caminho)
    url = card["source_zip"]
    if url not in arquivos:
        zip_path = Path(arquivos["_diretorio"]) / f"{len(arquivos)}.zip"
        with urlopen(url, timeout=240) as resposta, zip_path.open("wb") as saida:
            digest = hashlib.sha1()
            while parte := resposta.read(1024 * 1024):
                digest.update(parte)
                saida.write(parte)
        if digest.hexdigest() != card["archive_sha1"]:
            raise ValueError(f"SHA-1 divergente: {Path(caminho).name}")
        arquivos[url] = zip_path
    with ZipFile(arquivos[url]) as zipado:
        item = zipado.getinfo(card["archive_member"])
        if item.file_size > MAX_MEMBER_BYTES:
            raise ValueError(f"Membro muito grande: {Path(caminho).name}")
        conteudo = zipado.read(item)
    sha = hashlib.sha256(conteudo.replace(b"\r\n", b"\n")).hexdigest()
    if sha != card["cassette_sha256_lf"]:
        raise ValueError(f"SHA-256 divergente: {Path(caminho).name}")
    dados = json.loads(conteudo)
    if dados.get("schema_version") != 1 or dados.get("complete") is not True:
        raise ValueError(f"Schema/complete invalido: {Path(caminho).name}")
    interacoes = dados.get("interactions")
    if not isinstance(interacoes, list) or len(interacoes) != int(card["interactions"]):
        raise ValueError(f"Contagem divergente: {Path(caminho).name}")
    metodos = collections.Counter()
    acoes = collections.Counter()
    estruturas = collections.Counter()
    respostas_post_atribuicao = collections.Counter()
    formatos_redirect_atribuicao = collections.Counter()
    for interacao in interacoes:
        metodo, acao, estrutura = detalhes_requisicao(interacao.get("request"))
        metodos[metodo] += 1
        estruturas[estrutura] += 1
        if acao:
            acoes[(metodo, acao)] += 1
        if metodo == "POST" and acao == "procedimento_atribuicao_cadastrar":
            resposta = interacao.get("response") or {}
            status = resposta.get("status_code")
            respostas_post_atribuicao[status] += 1
            if status == 302:
                cabecalhos = resposta.get("headers") or {}
                destino = next(
                    (valor for nome, valor in cabecalhos.items()
                     if nome.lower() == "location" and isinstance(valor, str)),
                    "",
                )
                args = parse_qs(urlsplit(destino).query, keep_blank_values=True)
                acao_destino = args.get("acao", [])
                classe = (
                    acao_destino[0] if len(acao_destino) == 1 and
                    acao_destino[0] in ("arvore_visualizar", "principal") else "outra",
                    len(args.get("id_procedimento", [])) == 1
                    and args["id_procedimento"][0].isdigit(),
                    args.get("acao_origem") == ["procedimento_atribuicao_cadastrar"],
                )
                formatos_redirect_atribuicao[classe] += 1
    nome_card = Path(caminho).name
    if card['installation'] == 'unb' and 'bloco-incluir-anotar-retirar' in nome_card:
        _diagnostico_bloco_escolher(interacoes)
    # Nunca imprimir valores de URLs, corpos, cookies, tokens, hashes de sessao ou HTML.
    nome = Path(caminho).name
    print(f"PASS {card['installation']} {nome}: {len(interacoes)} interacoes; SHA-1/SHA-256 OK")
    print("METHODS", sorted(metodos.items()))
    print("ACTIONS", sorted((metodo, acao, n) for (metodo, acao), n in acoes.items()))
    if not acoes:
        print("ACTIONS_NOT_EXTRACTED; REQUEST_KEYS", sorted(estruturas.items()))
    if respostas_post_atribuicao:
        print("ATTRIBUTION_POST_STATUSES", sorted(respostas_post_atribuicao.items()))
        print("ATTRIBUTION_302_SHAPES", sorted(formatos_redirect_atribuicao.items()))
    return len(interacoes)




def _campos_por_nome(corpo):
    """Somente PRESENCA de campos permitidos: jamais registra conteudo bruto."""
    permitidos = (
        "selBloco", "hdnDocumentosItensSelecionados", "hdnDocumentosItens",
        "hdnDocumentosNroItens", "sbmIncluir", "chkDocumentosItem0",
    )
    if isinstance(corpo, dict):
        texto = " ".join(str(k) for k in corpo.keys())
        return [c for c in permitidos if c in corpo or c in texto]
    if isinstance(corpo, str):
        return [c for c in permitidos if re.search(r"(?:^|[&?])" + c + r"=", corpo)]
    return []


def _diagnostico_bloco_escolher(interacoes):
    """Diferencia metodos, aceite, navegacao e lacuna de prova sem imprimir segredos."""
    for i, item in enumerate(interacoes):
        req = item.get("request") or {}
        metodo, acao, _ = detalhes_requisicao(req)
        if acao != "bloco_escolher":
            continue
        resp = item.get("response") or {}
        cabecalhos = resp.get("headers") or {}
        destino = next(
            (valor for nome, valor in cabecalhos.items()
             if str(nome).lower() == "location" and isinstance(valor, str)),
            "",
        )
        argumentos_destino = parse_qs(urlsplit(destino).query)
        acao_destino = argumentos_destino.get("acao", [])
        acao_segura = (
            acao_destino[0] if len(acao_destino) == 1
            and re.fullmatch(r"[a-z_]{1,64}", acao_destino[0]) else None
        )
        dados = req.get("body", req.get("data", req.get("payload", None)))
        campos = _campos_por_nome(dados)
        anterior = detalhes_requisicao(interacoes[i - 1].get("request"))[:2] if i else None
        posterior = (
            detalhes_requisicao(interacoes[i + 1].get("request"))[:2]
            if i + 1 < len(interacoes) else None
        )
        corpo_resposta = resp.get("body", "")
        formulario = (
            isinstance(corpo_resposta, str)
            and "frmBlocoEscolher" in corpo_resposta
        )
        print(
            "BLOCO_ESCOLHER_SHAPE",
            "idx=", i + 1,
            "method=", metodo,
            "status=", resp.get("status_code"),
            "request_keys=", sorted(req.keys()),
            "body_kind=", type(dados).__name__,
            "allowed_form_keys=", campos,
            "response_keys=", sorted(resp.keys()),
            "response_form_marker=", formulario,
            "redirect_action=", acao_segura,
            "previous=", anterior,
            "next=", posterior,
        )

def argumentos():
    if len(sys.argv) > 1:
        return sys.argv[1:]
    if PEDIDO.exists():
        return [
            linha.strip() for linha in PEDIDO.read_text(encoding="utf-8").splitlines()
            if linha.strip() and not linha.lstrip().startswith("#")
        ]
    return os.environ.get("CASSETTE_CARDS", "").split()


def main():
    cards = argumentos()
    if not cards:
        raise SystemExit(
            "Informe cards via argumentos, CASSETTE_CARDS ou "
            ".github/cassettes-a-inspecionar.txt"
        )
    with tempfile.TemporaryDirectory() as pasta:
        arquivos = {"_diretorio": pasta}
        total = sum(verificar(caminho, arquivos) for caminho in cards)
    print(f"TOTAL_VERIFICADO {len(cards)} cassettes / {total} interacoes; "
          "nao e replay nem comprovacao de paridade IES/SEI")


if __name__ == "__main__":
    main()
