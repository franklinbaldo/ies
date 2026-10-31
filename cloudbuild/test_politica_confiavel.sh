#!/usr/bin/env bash
# Testa a fronteira de confiança do candidato Cloud Build sem rede ou segredos.
set -euo pipefail
cd "$(dirname "$0")/.."
origem="$(pwd)"
temporarios="$(mktemp -d)"
trap 'rm -rf "$temporarios"' EXIT
# Os comandos auditados são os executados inline pelo Cloud Build, não um
# script do checkout da PR. Marcadores delimitam apenas o bloco de pré-gate.
# Simula a expansão documentada do Cloud Build: $$ vira $ no script.
awk '/# INICIO_POLITICA_CONFIAVEL/{capturar=1;next} /# FIM_POLITICA_CONFIAVEL/{capturar=0} capturar {sub(/^        /,"");print}' cloudbuild/ies-rust-prek.yaml | sed 's/\$\$/\$/g' > "$temporarios/preflight.sh"
if [ ! -s "$temporarios/preflight.sh" ]; then
  echo 'RED: build não verifica política canônica independente da branch' >&2
  exit 1
fi
if ! grep -q '^serviceAccount: projects/virtual-computer-fsb/serviceAccounts/ies-ci@virtual-computer-fsb.iam.gserviceaccount.com$' cloudbuild/ies-rust-prek.yaml; then
  echo 'RED: build não declara serviceAccount auditada pelo trigger' >&2
  exit 1
fi
mkdir "$temporarios/correto" "$temporarios/politica-trocada" "$temporarios/toolchain-trocada" \
  "$temporarios/checker-trocado" "$temporarios/build-script-injetado" "$temporarios/workspace-trocado" "$temporarios/checker-link" \
  "$temporarios/cargo-config-injetado" "$temporarios/rustfmt-config-injetado"
for pasta in correto politica-trocada toolchain-trocada checker-trocado build-script-injetado workspace-trocado checker-link cargo-config-injetado rustfmt-config-injetado; do
  cp prek.toml rust-toolchain.toml Cargo.toml "$temporarios/$pasta/"
  cp -a xtask "$temporarios/$pasta/"
done
printf '\n# alteração da própria lista de gates no checkout não confiável\n' >> "$temporarios/politica-trocada/prek.toml"
printf '\n# toolchain não aprovado\n' >> "$temporarios/toolchain-trocada/rust-toolchain.toml"
# Os executaveis xtask implementam 3 dos 7 gates e devem permanecer imutaveis.
printf '\n// trocado para retornar OK sem verificar\n' >> "$temporarios/checker-trocado/xtask/src/bin/check_size.rs"
# Cargo executa automaticamente xtask/build.rs, mesmo sem editar Cargo.toml.
printf 'fn main() {}\n' > "$temporarios/build-script-injetado/xtask/build.rs"
# O manifesto raiz tambem decide quais binarios sao compilados com -p xtask.
printf '\n# workspace alterado pela PR\n' >> "$temporarios/workspace-trocado/Cargo.toml"
# Um symlink não pode escapar ao inventário de fontes aprovado.
ln -s src/bin/check_size.rs "$temporarios/checker-link/xtask/build.rs"
# A configuração Cargo local pode substituir o compilador por um executável
# controlado pela branch, mesmo que Cargo.toml e xtask sejam autênticos.
mkdir "$temporarios/cargo-config-injetado/.cargo"
printf '[build]\nrustc = "./executavel-da-pr"\n' > "$temporarios/cargo-config-injetado/.cargo/config.toml"
printf 'skip_children = true\n' > "$temporarios/rustfmt-config-injetado/.rustfmt.toml"
(cd "$temporarios/correto"; bash -eu "$temporarios/preflight.sh" >/dev/null)
for pasta in politica-trocada toolchain-trocada checker-trocado build-script-injetado workspace-trocado checker-link cargo-config-injetado rustfmt-config-injetado; do
  if (cd "$temporarios/$pasta"; bash -eu "$temporarios/preflight.sh" >/dev/null 2>&1); then
    echo "FAIL: aceitou configuração adulterada: $pasta" >&2
    exit 1
  fi
done
echo 'PASS: política, toolchain, workspace e xtask protegidos; adulterações recusadas'
