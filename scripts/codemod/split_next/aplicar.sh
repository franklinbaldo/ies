#!/bin/sh
# Aplica as regras, em ordem de nome, aos arquivos dados; cada regra repete
# até não mudar mais nada. Os arquivos são os de produção: as regras pulam
# módulos `#[cfg(test)]`, mas não sabem que um arquivo inteiro é de teste.
# uso: scripts/codemod/split_next/aplicar.sh [--dry-run] arquivo.rs...
set -eu
SG="uvx -q --from ast-grep-cli==0.51.2 ast-grep"
DIR=$(dirname "$0")
SECO=0
if [ "${1:-}" = "--dry-run" ]; then
    SECO=1
    shift
fi
# Regra que não se lê é erro, não silêncio.
for regra in "$DIR"/regras/*.yml; do
    if ! $SG scan -r "$regra" --json=stream /dev/null >/dev/null 2>&1; then
        echo "$(basename "$regra"): regra inválida" >&2
        exit 1
    fi
done
if [ "$SECO" = 1 ]; then
    for regra in "$DIR"/regras/*.yml; do
        $SG scan -r "$regra" --json=stream "$@"
    done
    exit 0
fi
for regra in "$DIR"/regras/*.yml; do
    passadas=0
    while $SG scan -r "$regra" -U "$@" 2>&1 | grep -q '^Applied'; do
        passadas=$((passadas + 1))
        if [ "$passadas" -ge 20 ]; then
            echo "$(basename "$regra"): não converge em 20 passadas" >&2
            exit 1
        fi
    done
done
