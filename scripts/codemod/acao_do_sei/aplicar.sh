#!/bin/sh
# Aplica as regras, em ordem, aos arquivos dados; cada regra repete até não
# mudar mais nada.
# Antes de tudo, a tabela valor -> variante sai de src/acao_do_sei.rs (só o
# enum AcaoDoSei), e cada modelo de modelos/ com VALOR e VARIANTE vira um
# YAML com um documento por linha da tabela. As funções com parâmetro
# `acao: &str` (que a regra 30 tipa) viram a lista FUNCOES das regras 31 e 32.
# Com --testes, só as regras 30 a 33, 89 e 90 (os arquivos de teste não têm
# `#[cfg(test)]` que as regras 10 a 24 saibam pular).
# uso: scripts/codemod/acao_do_sei/aplicar.sh [--dry-run] [--testes] arquivo.rs...
set -eu
SG="uvx -q --from ast-grep-cli==0.51.2 ast-grep"
DIR=$(dirname "$0")
ACOES="$DIR/../../../src/acao_do_sei.rs"
SECO=0
TESTES=0
while :; do
    case "${1:-}" in
        --dry-run) SECO=1; shift ;;
        --testes) TESTES=1; shift ;;
        *) break ;;
    esac
done
GERADAS=$(mktemp -d)
trap 'rm -rf "$GERADAS"' EXIT
# Tabela: uma linha "valor Variante" por variante de AcaoDoSei.
TABELA="$GERADAS/tabela"
awk '/^    AcaoDoSei \{/ { dentro = 1; next } dentro && /^    \}/ { dentro = 0 } dentro' "$ACOES" \
    | sed -n 's/^ *\([A-Z][A-Za-z0-9]*\) => "\([a-z0-9_]*\)",$/\2 \1/p' > "$TABELA"
if [ ! -s "$TABELA" ]; then
    echo "tabela vazia: $ACOES mudou de forma?" >&2
    exit 1
fi
# expandir modelo destino: um documento YAML por linha da tabela.
expandir() {
    primeiro=1
    while read -r valor variante; do
        [ "$primeiro" = 1 ] || echo '---'
        primeiro=0
        sed -e "s/VALOR/$valor/g" -e "s/VARIANTE/$variante/g" "$1"
    done < "$TABELA" > "$2"
}
if [ "$TESTES" = 1 ]; then
    cp "$DIR"/regras/3*.yml "$DIR"/regras/8*.yml "$DIR"/regras/9*.yml "$GERADAS"/
else
    cp "$DIR"/regras/*.yml "$GERADAS"/
    for modelo in 20-literal-comparado 21-unica-comparada-com-literal \
        22-lista-comparada-com-literal 23-literal-em-ramo 24-literal-na-url-formatada; do
        expandir "$DIR/modelos/$modelo.yml" "$GERADAS/$modelo.yml"
    done
fi
FUNCOES=$($SG scan -r "$DIR/modelos/funcoes-com-acao.yml" --json=stream "$@" \
    | python3 -I -c 'import sys, json; print("|".join(sorted({json.loads(l)["metaVariables"]["single"]["F"]["text"] for l in sys.stdin})))')
if [ -n "$FUNCOES" ]; then
    sed "s/FUNCOES/$FUNCOES/" "$DIR/modelos/31-argumento-como-na-url.yml" > "$GERADAS/31-argumento-como-na-url.yml"
    sed "s/FUNCOES/$FUNCOES/" "$DIR/modelos/32-argumento-literal.yml" > "$GERADAS/32-modelo"
    expandir "$GERADAS/32-modelo" "$GERADAS/32-argumento-literal.yml"
    rm "$GERADAS/32-modelo"
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
