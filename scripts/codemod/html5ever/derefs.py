"""Aplica a sugestão `*` do rustc onde um `&Elemento` chega a quem espera
`Elemento` (a regra 40 passou o parâmetro a valor; a fatia de um `Vec`
entrega referência). Só aceita a sugestão "consider dereferencing the
borrow" de um E0308 entre `Elemento` e `&Elemento`. Repete até o
compilador não sugerir mais nenhuma.

uso: python3 -I scripts/codemod/html5ever/derefs.py  (na raiz do repositório)
"""

import json
import subprocess
import sys


def sugestoes():
    saida = subprocess.run(
        ["cargo", "check", "--bin", "ies", "--tests", "--message-format=json", "-q"],
        capture_output=True,
        text=True,
        check=False,
    ).stdout
    achadas = {}
    for linha in saida.splitlines():
        try:
            item = json.loads(linha)
        except json.JSONDecodeError:
            continue
        mensagem = item.get("message") or {}
        codigo = (mensagem.get("code") or {}).get("code")
        texto = mensagem.get("rendered") or ""
        if codigo != "E0308" or "found `&" not in texto or "Elemento<" not in texto:
            continue
        for filho in mensagem.get("children", []):
            if "dereferencing the borrow" not in filho.get("message", ""):
                continue
            for trecho in filho.get("spans", []):
                if trecho.get("suggested_replacement") == "*":
                    chave = (trecho["file_name"], trecho["byte_start"])
                    achadas[chave] = trecho
    return achadas


def aplicar(achadas):
    por_arquivo = {}
    for (arquivo, inicio), trecho in achadas.items():
        por_arquivo.setdefault(arquivo, []).append(trecho)
    for arquivo, trechos in por_arquivo.items():
        with open(arquivo, "rb") as f:
            fonte = f.read()
        for trecho in sorted(trechos, key=lambda t: -t["byte_start"]):
            i, j = trecho["byte_start"], trecho["byte_end"]
            fonte = fonte[:i] + b"*" + fonte[j:]
        with open(arquivo, "wb") as f:
            f.write(fonte)
        print(arquivo, len(trechos))


def main():
    for _ in range(10):
        achadas = sugestoes()
        if not achadas:
            return 0
        aplicar(achadas)
    print("derefs: não converge", file=sys.stderr)
    return 1


if __name__ == "__main__":
    sys.exit(main())
