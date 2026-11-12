"""Tira os `use` que o rustc aponta como sem uso (lint `unused_imports`),
aplicando só a sugestão dele, que é exata. O `cargo fix` aplicaria também
as outras sugestões; aqui só os imports saem. Repete até não sobrar nenhum.

uso: python3 -I scripts/codemod/html5ever/imports_sem_uso.py  (na raiz)
"""

import json
import subprocess
import sys


def trechos():
    saida = subprocess.run(
        ["cargo", "check", "--bin", "ies", "--tests", "--message-format=json", "-q"],
        capture_output=True,
        text=True,
        check=False,
    ).stdout
    achados = {}
    for linha in saida.splitlines():
        try:
            item = json.loads(linha)
        except json.JSONDecodeError:
            continue
        mensagem = item.get("message") or {}
        if (mensagem.get("code") or {}).get("code") != "unused_imports":
            continue
        for filho in mensagem.get("children", []):
            for trecho in filho.get("spans", []):
                if trecho.get("suggested_replacement") is None:
                    continue
                chave = (trecho["file_name"], trecho["byte_start"], trecho["byte_end"])
                achados[chave] = trecho["suggested_replacement"]
    return achados


def main():
    for _ in range(5):
        achados = trechos()
        if not achados:
            return 0
        por_arquivo = {}
        for (arquivo, inicio, fim), novo in achados.items():
            por_arquivo.setdefault(arquivo, []).append((inicio, fim, novo))
        for arquivo, lista in por_arquivo.items():
            with open(arquivo, "rb") as f:
                fonte = f.read()
            ultimo = len(fonte) + 1
            for inicio, fim, novo in sorted(lista, reverse=True):
                if fim > ultimo:
                    continue
                fonte = fonte[:inicio] + novo.encode() + fonte[fim:]
                ultimo = inicio
            with open(arquivo, "wb") as f:
                f.write(fonte)
    print("imports_sem_uso: não converge", file=sys.stderr)
    return 1


if __name__ == "__main__":
    sys.exit(main())
