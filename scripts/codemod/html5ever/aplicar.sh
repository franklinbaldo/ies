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
# Regra que não se lê é erro, não silêncio.
for regra in "$DIR"/regras/*.yml; do
    if ! $SG scan -r "$regra" --json=stream /dev/null >/dev/null 2>&1; then
        echo "$(basename "$regra"): regra inválida" >&2
        exit 1
    fi
done
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
# Uma regra pode abrir caminho para uma anterior (a 18 troca atributos() por
# atributo(), que a 11 reconhece): a sequência inteira repete até uma
# rodada não mudar nada, no máximo 5 rodadas.
rodada=0
while :; do
    mudou=0
    for regra in "$GERADAS"/*.yml; do
        passadas=0
        while $SG scan -r "$regra" -U "$@" 2>&1 | grep -q '^Applied'; do
            mudou=1
            passadas=$((passadas + 1))
            if [ "$passadas" -ge 20 ]; then
                echo "$(basename "$regra"): não converge em 20 passadas" >&2
                exit 1
            fi
        done
    done
    rodada=$((rodada + 1))
    if [ "$mudou" = 0 ]; then
        break
    fi
    if [ "$rodada" -ge 5 ]; then
        echo "as regras não convergem em 5 rodadas" >&2
        exit 1
    fi
done
sed -i -e '/^[[:space:]]*\/\* ies-codemod: remover linha \*\/[[:space:]]*$/d' \
    -e 's/ *\/\* ies-codemod: remover linha \*\/ *//g' "$@"
