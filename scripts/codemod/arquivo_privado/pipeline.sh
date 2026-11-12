#!/bin/sh
# Refaz, a partir de qualquer HEAD limpo, a troca das variantes
# ArquivoPrivado* de FalhaTransporte por ErroArquivoPrivado, o fim de
# CookieIlegivel e CredenciaisIlegiveis e o cookie dos comandos vindo da
# abertura. Etapas (README.md):
#   0. o modelo da leitura ainda reproduz a função antiga;
#   1. regra 05: configuracao::ErroArquivoPrivado vira ErroArquivoDeAcesso;
#   2. regras 1x: ler_cookie(..)? vira o cookie da abertura; Cookie sai;
#   3. regras 3x: a leitura sai de transporte e as variantes mudam de tipo;
#   4. regras 4x: os testes da CLI sem sessão conferem o novo erro;
#   5. modelos/leitura.rs vira src/arquivo_privado/leitura.rs;
#   6. ajustes.py (ajustes_*.py): o que nenhuma regra cobre, com o motivo;
#   7. imports_sem_uso.py e cargo fmt.
# Com --ate-as-regras, para antes do passo 6 (o estado de que ajustes.py
# parte; serve para refazer um ajuste que deixou de casar).
# uso: scripts/codemod/arquivo_privado/pipeline.sh [--ate-as-regras]  (na raiz, árvore limpa)
set -eu
ATE_AS_REGRAS=0
if [ "${1:-}" = "--ate-as-regras" ]; then
    ATE_AS_REGRAS=1
fi
SG="uvx -q --from ast-grep-cli==0.51.2 ast-grep"
DIR=$(dirname "$0")
ALVOS="src"

if [ -n "$(git status --porcelain -- src tests docs contracts)" ]; then
    echo "pipeline: src, tests, docs ou contracts com mudança; rode numa árvore limpa" >&2
    exit 1
fi

# Regra que não se lê é erro, não silêncio.
for regra in "$DIR"/regras/*.yml "$DIR"/modelos/*.yml; do
    if ! $SG scan -r "$regra" --json=stream /dev/null >/dev/null 2>&1; then
        echo "$(basename "$regra"): regra inválida" >&2
        exit 1
    fi
done

aplicar() {
    regra=$1
    alvos=${2:-$ALVOS}
    passadas=0
    while $SG scan -r "$regra" -U $alvos 2>&1 | grep -q '^Applied'; do
        passadas=$((passadas + 1))
        if [ "$passadas" -ge 20 ]; then
            echo "$(basename "$regra"): não converge em 20 passadas" >&2
            exit 1
        fi
    done
}

remover_marcas() {
    grep -rlF '/* ies-codemod: remover linha */' $ALVOS | while read -r arquivo; do
        sed -i -e '/^[[:space:]]*\(\/\* ies-codemod: remover linha \*\/[[:space:]]*\)\{1,\}$/d' \
            -e 's/ *\/\* ies-codemod: remover linha \*\/ *//g' "$arquivo"
    done
}

# modelos/leitura.rs reproduz a função antiga: se ela mudou no HEAD, o
# modelo precisa da mesma mudança antes (e arquivo_privado_antigo.rs.txt).
ANTIGA=$($SG scan -r "$DIR/modelos/funcao-antiga.yml" --json=stream src/transporte.rs \
    | python3 -I -c 'import sys, json; [sys.stdout.write(json.loads(l)["text"] + "\n") for l in sys.stdin]')
if [ "$ANTIGA" != "$(cat "$DIR/modelos/arquivo_privado_antigo.rs.txt")" ]; then
    echo "pipeline: transporte::arquivo_privado mudou; atualize modelos/leitura.rs" >&2
    exit 1
fi

# 1: o nome ErroArquivoPrivado de configuracao (metadados dos arquivos de
# acesso) vira ErroArquivoDeAcesso, antes de o nome passar à leitura.
aplicar "$DIR/regras/05-erro-de-acesso.yml" "src/configuracao.rs src/configuracao"

# 2: o cookie dos comandos vem da abertura; a variante Cookie sai.
for regra in "$DIR"/regras/1*.yml; do
    aplicar "$regra"
done
remover_marcas

# 3: a leitura sai de transporte e as variantes mudam de tipo
for regra in "$DIR"/regras/3*.yml; do
    aplicar "$regra"
done
remover_marcas
# 4: os testes da CLI que conferem o erro de quem roda sem sessão
for regra in "$DIR"/regras/4*.yml; do
    aplicar "$regra" tests
done

# 5 e 6
cp "$DIR/modelos/leitura.rs" src/arquivo_privado/leitura.rs
cargo fmt --all
if [ "$ATE_AS_REGRAS" = 1 ]; then
    exit 0
fi
python3 -I "$DIR/ajustes.py"

# 7
cargo fmt --all
python3 -I scripts/codemod/html5ever/imports_sem_uso.py
cargo fmt --all
