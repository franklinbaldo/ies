#!/usr/bin/env bash
# Teste offline de integridade: snapshot Cloud Build nao pode perder arquivos
# de uma PR quando .gitignore muda; nada e escrito fora da pasta temporaria.
set -euo pipefail
cd "$(dirname "$0")/.."
temporarios="$(mktemp -d)"
trap 'rm -rf "$temporarios"' EXIT
awk '
  /if ! git rev-parse --is-inside-work-tree/ {capturar=1}
  capturar {
    sub(/^        /, "")
    print
    if ($0 == "fi") exit
  }
' cloudbuild/ies-rust-prek.yaml > "$temporarios/indexar.sh"
if ! grep -q 'git init --quiet' "$temporarios/indexar.sh"; then
  echo 'FAIL: nao foi localizado o bootstrap de indice do build' >&2
  exit 1
fi
mkdir "$temporarios/snapshot"
printf 'artefato-de-contrato.txt\n' > "$temporarios/snapshot/.gitignore"
printf 'conteudo que nao pode escapar ao gate de tamanho\n' > "$temporarios/snapshot/artefato-de-contrato.txt"
(
  cd "$temporarios/snapshot"
  bash -e "$temporarios/indexar.sh"
  if ! git ls-files --error-unmatch artefato-de-contrato.txt >/dev/null 2>&1; then
    echo 'FAIL: snapshot perdeu arquivo versionado por obedecer .gitignore da PR' >&2
    exit 1
  fi
)
echo 'PASS: arquivos do snapshot sao indexados mesmo que .gitignore os oculte'
