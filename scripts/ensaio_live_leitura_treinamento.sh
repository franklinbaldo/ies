#!/usr/bin/env bash
# Read-only live smoke test: public training installations only.
set -euo pipefail
umask 077
bin="${1:-target/release/ies}"
for key in unb pen1 ufrj go; do
  d="$(mktemp -d)"
  export IES_ORIGEM_FILE="$d/origem" IES_COOKIE_FILE="$d/cookie"
  export IES_URL_CONTROLE_FILE="$d/controle" IES_CREDENCIAIS_FILE="$d/credenciais"
  export IES_SEM_INDICE=1
  unset IES_CONFIG_FILE
  python3 -I - "$key" "$d" <<'PY'
import sys
from pathlib import Path
key, directory = sys.argv[1:]
rows = Path("docs/interfaces/credenciais-de-treinamento.md").read_text().splitlines()
found = []
for line in rows:
    if not line.startswith("| "):
        continue
    cols = [c.strip().strip(chr(96)) for c in line.strip("|").split("|")]
    if cols[0] == key:
        found.append(cols)
if len(found) != 1:
    raise SystemExit("training account not found")
cols = found[0]
Path(directory, "origem").write_text(cols[2] + "\n")
Path(directory, "credenciais").write_text(
    "usuario=" + cols[3] + "\nsenha=" + cols[4] + "\norgao=" + cols[5] + "\n"
)
PY
  chmod 600 "$d/origem" "$d/credenciais"
  set +e
  "$bin" --json login > "$d/login.json" 2>"$d/login.err"
  login_rc=$?
  set -e
  python3 -I - "$key" "$login_rc" "$d/login.json" <<'PY'
import json,sys
key,rc,path=sys.argv[1:]
try: result=json.load(open(path))
except (OSError,ValueError): result={}
print("LIVE",key,"login",rc,{k:result[k] for k in
 ("conexao","requisicoes","unidade_exibida","subcodigo","status_http")
 if k in result})
PY
  if [ "$login_rc" -eq 0 ]; then
    sleep 35
    set +e
    "$bin" --json unidade atual > "$d/unidade.json" 2>"$d/unidade.err"
    unidade_rc=$?
    set -e
    python3 -I - "$key" "$unidade_rc" "$d/unidade.json" <<'PY'
import json,sys
key,rc,path=sys.argv[1:]
try: result=json.load(open(path))
except (OSError,ValueError): result={}
print("LIVE",key,"unidade_atual",rc,{k:result[k] for k in
 ("unidade_exibida","requisicoes","subcodigo","status_http")
 if k in result})
PY
  fi
  rm -rf "$d"
  unset IES_ORIGEM_FILE IES_COOKIE_FILE IES_URL_CONTROLE_FILE IES_CREDENCIAIS_FILE
  sleep 35
done
