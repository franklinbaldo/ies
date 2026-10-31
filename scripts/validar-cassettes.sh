#!/usr/bin/env bash
# Validação offline: não abre sessão, não grava cassettes, não publica nem faz GET/POST.
set -euo pipefail
usage() {
  cat <<'EOF'
Uso: scripts/validar-cassettes.sh --acervo DIR --bin EXECUTAVEL
     --cenario unb:NOME [--cenario ufrj:NOME ...] [--executar]
Sem --executar: prévia sem comandos de IES e sem rede.
Com --executar: replays offline em paralelo, ZIPs já presentes no cache local.
EOF
}
acervo=''
bin=''
executar=0
declare -a cenarios=()
while (($#)); do
  case "$1" in
    --help|-h) usage; exit 0 ;;
    --acervo|--bin|--cenario)
      (($# >= 2)) || { echo "Falta valor para $1" >&2; exit 2; }
      case "$1" in
        --acervo) acervo=$2 ;;
        --bin) bin=$2 ;;
        --cenario) cenarios+=("$2") ;;
      esac
      shift 2 ;;
    --executar) executar=1; shift ;;
    *) echo "Opção não reconhecida" >&2; exit 2 ;;
  esac
done
[[ -n "$acervo" && -n "$bin" && ${#cenarios[@]} -gt 0 ]] || { usage >&2; exit 2; }
for spec in "${cenarios[@]}"; do
  [[ "$spec" == *:* ]] || { echo "Cenário deve ter instalação explícita" >&2; exit 2; }
  inst=${spec%%:*}
  cenario=${spec#*:}
  case "$inst" in unb|ufrj|go|pen1|pen2|pen3|pen4) ;; *) echo "Instalação inválida" >&2; exit 2 ;; esac
  [[ "$cenario" =~ ^[a-zA-Z0-9_-]+$ ]] || { echo "Nome de cenário inválido" >&2; exit 2; }
done
if (( ! executar )); then
  printf 'NOT_RUN (prévia; sem rede): %s\n' "${cenarios[@]}"
  exit 0
fi
[[ -d "$acervo/cassettes" && -x "$bin" ]] || { echo "Acervo ou executável ausente" >&2; exit 2; }
umask 077
temporarios=$(mktemp -d)
trap 'rm -rf -- "$temporarios"' EXIT
declare -a pids=()
for i in "${!cenarios[@]}"; do
  (
    spec=${cenarios[i]}
    inst=${spec%%:*}
    cenario=${spec#*:}
    if "$bin" replay "$cenario" --acervo "$acervo" --json >"$temporarios/$i.json" 2>"$temporarios/$i.err"; then
      if python3 - "$temporarios/$i.json" "$inst" <<'PY'
import json
import sys
with open(sys.argv[1], encoding="utf-8") as handle:
    data = json.load(handle)
# Identidade da cassette verificada pelo IES: não expor corpo nem cookies.
if data.get("codigo"):
    sys.exit(1)
instalacao = data.get("instalacao", data.get("origem"))
if instalacao != sys.argv[2]:
    sys.exit(1)
# Replay de caixa em testes de aceite deve cobrir todos os totais declarados.
for grupo in ("recebidos", "gerados"):
    if grupo in data:
        tabela = data[grupo]
        total = tabela.get("total_declarado")
        if total is None or len(tabela.get("processos", [])) != total:
            sys.exit(1)
PY
      then
        printf 'PASS %s (replay offline verificado)\n' "$spec" >"$temporarios/$i.result"
      else
        printf 'FAIL %s (instalação divergente ou JSON inválido)\n' "$spec" >"$temporarios/$i.result"
        exit 1
      fi
    else
      printf 'FAIL %s (replay recusado; verificar ZIP e card localmente)\n' "$spec" >"$temporarios/$i.result"
      exit 1
    fi
  ) &
  pids+=("$!")
done
falhas=0
for pid in "${pids[@]}"; do
  if ! wait "$pid"; then falhas=1; fi
done
for i in "${!cenarios[@]}"; do
  cat "$temporarios/$i.result"
done
exit "$falhas"
