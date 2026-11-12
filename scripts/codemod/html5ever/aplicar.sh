#!/bin/sh
# Aplica as regras, em ordem, aos arquivos dados; cada regra repete até
# não mudar mais nada (as aninhadas saem uma camada por passada). As regras
# que apagam um comando deixam a marca `/* ies-codemod: remover linha */`,
# e a linha inteira sai no fim.
# uso: scripts/codemod/html5ever/aplicar.sh [--dry-run] arquivo.rs...
set -eu
SG="uvx -q --from ast-grep-cli==0.51.2 ast-grep"
REGRAS=$(dirname "$0")/regras
if [ "${1:-}" = "--dry-run" ]; then
    shift
    for regra in "$REGRAS"/*.yml; do
        $SG scan -r "$regra" --json=stream "$@"
    done
    exit 0
fi
for regra in "$REGRAS"/*.yml; do
    while $SG scan -r "$regra" -U "$@" 2>&1 | grep -q '^Applied'; do :; done
done
sed -i '/^[[:space:]]*\/\* ies-codemod: remover linha \*\/[[:space:]]*$/d' "$@"
