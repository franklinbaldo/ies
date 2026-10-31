#!/usr/bin/env bash
# Testa a fronteira de confiança do candidato Cloud Build sem rede ou segredos.
set -euo pipefail
cd "$(dirname "$0")/.."
origem="$(pwd)"
temporarios="$(mktemp -d)"
trap 'rm -rf "$temporarios"' EXIT
# Os comandos auditados são os executados inline pelo Cloud Build, não um
# script do checkout da PR. Marcadores delimitam apenas o bloco de pré-gate.
awk '/# INICIO_POLITICA_CONFIAVEL/{capturar=1;next} /# FIM_POLITICA_CONFIAVEL/{capturar=0} capturar {sub(/^        /,"");print}' cloudbuild/ies-rust-prek.yaml > "$temporarios/preflight.sh"
if [ ! -s "$temporarios/preflight.sh" ]; then
  echo 'RED: build não verifica política canônica independente da branch' >&2
  exit 1
fi
if ! grep -q '^serviceAccount: projects/virtual-computer-fsb/serviceAccounts/ies-ci@virtual-computer-fsb.iam.gserviceaccount.com$' cloudbuild/ies-rust-prek.yaml; then
  echo 'RED: build não declara serviceAccount auditada pelo trigger' >&2
  exit 1
fi
mkdir "$temporarios/correto" "$temporarios/politica-trocada" "$temporarios/toolchain-trocada"
for pasta in correto politica-trocada toolchain-trocada; do
  cp prek.toml rust-toolchain.toml "$temporarios/$pasta/"
done
printf '\n# alteração da própria lista de gates no checkout não confiável\n' >> "$temporarios/politica-trocada/prek.toml"
printf '\n# toolchain não aprovado\n' >> "$temporarios/toolchain-trocada/rust-toolchain.toml"
(cd "$temporarios/correto"; bash -eu "$temporarios/preflight.sh" >/dev/null)
for pasta in politica-trocada toolchain-trocada; do
  if (cd "$temporarios/$pasta"; bash -eu "$temporarios/preflight.sh" >/dev/null 2>&1); then
    echo "FAIL: aceitou configuração adulterada: $pasta" >&2
    exit 1
  fi
done
echo 'PASS: política canônica protegida; alterações em prek/toolchain recusadas'
