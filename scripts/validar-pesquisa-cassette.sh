#!/usr/bin/env bash
# Somente replay offline verificado: nunca conecta ao SEI ou publica capturas.
set -euo pipefail
uso() {
  echo 'Uso: validar-pesquisa-cassette.sh --acervo DIR --bin EXEC --instalacao unb|ufrj|go|pen1|pen2|pen3|pen4 --cenario CARD --passo N --numero-processo NUP [--executar]'
}
acervo='' executavel='' instalacao='' cenario='' passo='' numero='' executar=0
while (($#)); do
  case "$1" in
    --acervo|--bin|--instalacao|--cenario|--passo|--numero-processo)
      (($# >= 2)) || { uso >&2; exit 2; }
      case "$1" in
        --acervo) acervo=$2 ;;
        --bin) executavel=$2 ;;
        --instalacao) instalacao=$2 ;;
        --cenario) cenario=$2 ;;
        --passo) passo=$2 ;;
        --numero-processo) numero=$2 ;;
      esac
      shift 2 ;;
    --executar) executar=1; shift ;;
    --help|-h) uso; exit 0 ;;
    *) echo 'Opção desconhecida' >&2; uso >&2; exit 2 ;;
  esac
done
[[ -n "$acervo" && -n "$executavel" && -n "$cenario" && -n "$numero" ]] || { uso >&2; exit 2; }
case "$instalacao" in unb|ufrj|go|pen1|pen2|pen3|pen4) ;; *) echo 'Instalação não permitida' >&2; exit 2;; esac
[[ "$passo" =~ ^(0|[1-9][0-9]*)$ && "$cenario" =~ ^[a-zA-Z0-9_-]+$ ]] || { echo 'Passo ou card inválido' >&2; exit 2; }
if (( ! executar )); then
  printf 'NOT_RUN: prévia offline para %s, passo %s; sem acesso ao SEI\n' "$instalacao" "$passo"
  exit 0
fi
[[ -x "$executavel" && -d "$acervo/cassettes" ]] || { echo 'Binário ou acervo ausente' >&2; exit 2; }
umask 077
tmp=$(mktemp -d)
trap 'rm -rf -- "$tmp"' EXIT
if ! "$executavel" --json replay "$cenario" --acervo "$acervo" --resultado-pesquisa \
    --passo "$passo" --numero-processo "$numero" >"$tmp/saida.json" 2>"$tmp/erro.txt"; then
  echo 'FAIL: replay recusado; verificar ZIP, card, resposta e NUP localmente' >&2
  exit 1
fi
if python3 - "$tmp/saida.json" "$instalacao" "$numero" <<'PY'
import json
import sys
with open(sys.argv[1], encoding='utf-8') as handle:
    d = json.load(handle)
assert d.get('instalacao') == sys.argv[2]
assert d.get('escopo') == 'resultado_pesquisa_cassette_offline'
assert d.get('numero_solicitado') == sys.argv[3]
assert isinstance(d.get('total_observado'), int) and d['total_observado'] >= 1
assert d.get('processos') == [{'numero': sys.argv[3]}]
assert 'id_procedimento' not in json.dumps(d)
PY
then
  printf 'PASS: NUP observado exatamente na resposta offline verificada (%s)\n' "$instalacao"
else
  echo 'FAIL: identidade, escopo ou formato não conferidos' >&2
  exit 1
fi
