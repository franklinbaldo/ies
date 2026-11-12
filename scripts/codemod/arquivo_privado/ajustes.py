"""Ajustes que nenhuma regra cobre, cada um com o motivo, em
ajustes_codigo.py, ajustes_testes.py e ajustes_docs.py. Roda depois das
regras (pipeline.sh, passo 6), na raiz. Cada troca exige o trecho velho
exatamente uma vez (ou o número dito): se um trecho mudou no HEAD, o
script para e diz qual, sem aplicar meia troca no arquivo.

uso: python3 -I scripts/codemod/arquivo_privado/ajustes.py  (na raiz)
"""

import importlib
import os
import sys

sys.dont_write_bytecode = True
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
AJUSTES = importlib.import_module("ajustes_base").AJUSTES
importlib.import_module("ajustes_codigo")
importlib.import_module("ajustes_cookie")
importlib.import_module("ajustes_docs")
modulos_de_apoio = importlib.import_module("ajustes_testes").modulos_de_apoio


def main():
    falhas = 0
    por_arquivo = {}
    for arquivo, motivo, funcao in AJUSTES:
        por_arquivo.setdefault(arquivo, []).append((motivo, funcao))
    for arquivo, lista in por_arquivo.items():
        with open(arquivo, encoding="utf-8") as f:
            texto = f.read()
        try:
            for motivo, funcao in lista:
                texto = funcao(texto)
        except ValueError as erro:
            print(f"{arquivo}: {motivo}: {erro}", file=sys.stderr)
            falhas += 1
            continue
        with open(arquivo, "w", encoding="utf-8") as f:
            f.write(texto)
    if falhas:
        return 1
    modulos_de_apoio()
    return 0


if __name__ == "__main__":
    sys.exit(main())
