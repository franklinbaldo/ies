#!/bin/sh
# Aplica as regras, em ordem, aos arquivos dados; cada regra repete até
# não mudar mais nada (as aninhadas saem uma camada por passada). As regras
# que apagam um comando deixam a marca `/* ies-codemod: remover linha */`,
# e a linha inteira sai no fim.
# Antes de tudo, as funções com parâmetro `&tl::Parser` (que a regra 42
# remove) viram a lista da regra 43, gerada de `modelos/`, que tira o
# argumento correspondente nas chamadas.
# uso: scripts/codemod/html5ever/aplicar.sh [--dry-run] arquivo.rs...
set -eu
SG="uvx -q --from ast-grep-cli==0.51.2 ast-grep"
DIR=$(dirname "$0")
SECO=0
if [ "${1:-}" = "--dry-run" ]; then
    SECO=1
    shift
fi
GERADAS=$(mktemp -d)
trap 'rm -rf "$GERADAS"' EXIT
cp "$DIR"/regras/*.yml "$GERADAS"/
FUNCOES=$($SG scan -r "$DIR/modelos/funcoes-com-parser.yml" --json=stream "$@" \
    | python3 -I -c 'import sys, json; print("|".join(sorted({json.loads(l)["metaVariables"]["single"]["F"]["text"] for l in sys.stdin})))')
if [ -n "$FUNCOES" ]; then
    sed "s/FUNCOES/$FUNCOES/" "$DIR/modelos/43-argumento-parser.yml" > "$GERADAS/43-argumento-parser.yml"
fi
COM_ELEMENTO=$($SG scan -r "$DIR/modelos/funcoes-com-elemento.yml" --json=stream "$@" \
    | python3 -I -c 'import sys, json; print("|".join(sorted({json.loads(l)["metaVariables"]["single"]["F"]["text"] for l in sys.stdin})))')
if [ -n "$COM_ELEMENTO" ]; then
    for modelo in 44-copia-para-funcao 44b-funcao-direta; do
        sed "s/FUNCOES/$COM_ELEMENTO/" "$DIR/modelos/$modelo.yml" > "$GERADAS/$modelo.yml"
    done
fi
if [ "$SECO" = 1 ]; then
    for regra in "$GERADAS"/*.yml; do
        $SG scan -r "$regra" --json=stream "$@"
    done
    exit 0
fi
for regra in "$GERADAS"/*.yml; do
    passadas=0
    while $SG scan -r "$regra" -U "$@" 2>&1 | grep -q '^Applied'; do
        passadas=$((passadas + 1))
        if [ "$passadas" -ge 20 ]; then
            echo "$(basename "$regra"): não converge em 20 passadas" >&2
            exit 1
        fi
    done
done
sed -i -e '/^[[:space:]]*\/\* ies-codemod: remover linha \*\/[[:space:]]*$/d' \
    -e 's/ *\/\* ies-codemod: remover linha \*\/ *//g' "$@"
