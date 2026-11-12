#!/bin/sh
# Aplica as regras de reescrita (10-*.yml), em ordem, aos arquivos dados; cada
# regra repete até não mudar mais nada. Depois lista, sem mudar, os sítios da
# regra 90 (closure que ignora o erro e não devolve variante unitária), que
# ficam para ajuste à mão.
# Com --dry-run, só lista o que cada regra reescreveria ou apontaria.
# uso: scripts/codemod/causa_do_erro/aplicar.sh [--dry-run] arquivo.rs...
set -eu
SG="uvx -q --from ast-grep-cli==0.51.2 ast-grep"
DIR=$(dirname "$0")
SECO=0
if [ "${1:-}" = "--dry-run" ]; then
    SECO=1
    shift
fi
listar() {
    regra=$1
    shift
    $SG scan -r "$regra" --json=stream "$@" | python3 -I -c '
import json, sys
for linha in sys.stdin:
    d = json.loads(linha)
    print("{}\t{}:{}".format(d["ruleId"], d["file"], d["range"]["start"]["line"] + 1))
'
}
if [ "$SECO" = 1 ]; then
    for regra in "$DIR"/regras/*.yml; do
        listar "$regra" "$@"
    done
    exit 0
fi
for regra in "$DIR"/regras/1*.yml; do
    passadas=0
    while $SG scan -r "$regra" -U "$@" 2>&1 | grep -q '^Applied'; do
        passadas=$((passadas + 1))
        if [ "$passadas" -ge 20 ]; then
            echo "$(basename "$regra"): não converge em 20 passadas" >&2
            exit 1
        fi
    done
done
for regra in "$DIR"/regras/9*.yml; do
    listar "$regra" "$@"
done
