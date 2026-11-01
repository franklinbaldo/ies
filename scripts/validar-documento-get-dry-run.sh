#!/bin/sh
# Roteiro SEM conexão: não lê credenciais, links assinados ou HTML autenticado.
# Execução real exige autorização por instalação/efeito e operador separado.
set -eu
if [ "$#" -ne 0 ]; then
    printf '%s\n' 'Uso: IES_INSTALACAO=unb IES_DOCUMENTO=0007 sh scripts/validar-documento-get-dry-run.sh' >&2
    exit 2
fi
case "${IES_INSTALACAO:-}" in
    unb|ufrj|go|pen) ;;
    *) printf '%s\n' 'Informe IES_INSTALACAO=unb|ufrj|go|pen (treinamento somente)' >&2; exit 2 ;;
esac
case "${IES_DOCUMENTO:-}" in
    ''|*[!0-9]*) printf '%s\n' 'Informe IES_DOCUMENTO: numero SEI ASCII positivo, zeros preservados' >&2; exit 2 ;;
esac
printf '%s\n' 'DRY_RUN / NOT_RUN: NENHUMA chamada HTTP, login, leitura de arquivo ou escrita.'
printf '%s\n' '1. Conferir domínio oficial de treinamento e condições de uso da instalação indicada.'
printf '%s\n' '2. Conferir autorização específica para GET documento_visualizar e eventual efeito administrativo.'
printf '%s\n' '3. Em sessão já autorizada, obter link fresco da própria árvore; nunca interpolar infra_hash.'
printf '%s\n' '4. Guardar URL assinada, cookie e origem exclusivamente em arquivos privados (0600).'
printf '%s\n' '5. Testar uma única consulta de documento, sem GET de editor, POST ou retry.'
printf '%s\n' '6. Conferir número DOC exibido, status/MIME, corpo, identidade de sessão e efeitos.'
printf '%s\n' '7. Registrar versão SEI, unidade, latência, contagem de requests e desfecho PASS/FAIL.'
printf '%s\n' '8. Redigir cassette offline apenas após remoção de cookies, hashes, links e conteúdo sensível.'
printf '%s\n' '9. Repetir somente nas instalações independentes efetivamente permitidas; sem concorrência.'
printf '%s\n' 'Este roteiro NÃO prova acesso live nem concede autorização.'
