#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
cat > "$tmp/valido.json" <<'JSON'
{
  "name": "ies-pr-ci",
  "repositoryEventConfig": {
    "repository": "projects/virtual-computer-fsb/locations/us-central1/connections/judicial-github/repositories/ies",
    "repositoryType": "GITHUB",
    "pullRequest": {"branch": ".*", "commentControl": "COMMENTS_ENABLED_FOR_EXTERNAL_CONTRIBUTORS_ONLY"}
  },
  "serviceAccount": "projects/virtual-computer-fsb/serviceAccounts/ies-ci@virtual-computer-fsb.iam.gserviceaccount.com",
  "build": {
    "serviceAccount": "projects/virtual-computer-fsb/serviceAccounts/ies-ci@virtual-computer-fsb.iam.gserviceaccount.com",
    "steps": [{"id": "uv-pinned", "name": "sha256:imutable"}, {"id": "rust-prek", "name": "sha256:imutable", "args": ["uvx --from prek==0.5.7 prek run --hook-stage pre-push --all-files"]}],
    "options": {"logging": "CLOUD_LOGGING_ONLY"},
    "timeout": "1200s"
  }
}
JSON
aprovado="$(jq -cS '.build | {steps, options, timeout, serviceAccount}' "$tmp/valido.json" | sha256sum | cut -d' ' -f1)"
bash ./auditar-trigger.sh "$tmp/valido.json" "$aprovado" >/dev/null
recusar() {
  local arquivo="$1" hash="$2"
  if bash ./auditar-trigger.sh "$arquivo" "$hash" >/dev/null 2>&1; then
    echo "FAIL: trigger inseguro aceito: $arquivo" >&2
    exit 1
  fi
}
jq '.filename = "cloudbuild/ies-rust-prek.yaml"' "$tmp/valido.json" > "$tmp/filename.json"
recusar "$tmp/filename.json" "$aprovado"
jq '.gitFileSource = {"path":"cloudbuild/ies-rust-prek.yaml"}' "$tmp/valido.json" > "$tmp/git-file.json"
recusar "$tmp/git-file.json" "$aprovado"
jq 'del(.repositoryEventConfig.pullRequest)' "$tmp/valido.json" > "$tmp/sem-pr.json"
recusar "$tmp/sem-pr.json" "$aprovado"
jq '.serviceAccount = "projects/virtual-computer-fsb/serviceAccounts/admin@virtual-computer-fsb.iam.gserviceaccount.com"' "$tmp/valido.json" > "$tmp/sa.json"
recusar "$tmp/sa.json" "$aprovado"
jq '.build.steps[1].args = ["echo OK; exit 0"]' "$tmp/valido.json" > "$tmp/gate-adulterado.json"
recusar "$tmp/gate-adulterado.json" "$aprovado"
recusar "$tmp/valido.json" "0000000000000000000000000000000000000000000000000000000000000000"
jq '.repositoryEventConfig.repository = "projects/virtual-computer-fsb/locations/us-central1/connections/judicial-github/repositories/outro"' "$tmp/valido.json" > "$tmp/outro-repo.json"
recusar "$tmp/outro-repo.json" "$aprovado"
jq '.repositoryEventConfig.pullRequest.branch = "refs/heads/nao-existe"' "$tmp/valido.json" > "$tmp/nao-roda-main.json"
recusar "$tmp/nao-roda-main.json" "$aprovado"
jq '.ignoredFiles = ["src/**", "contracts/**"]' "$tmp/valido.json" > "$tmp/ignora-codigo.json"
recusar "$tmp/ignora-codigo.json" "$aprovado"
jq '.disabled = true' "$tmp/valido.json" > "$tmp/desativado.json"
recusar "$tmp/desativado.json" "$aprovado"
echo "PASS: fonte inline de confiança, SHA de gate, identidade e PR verificados; dez cenários negativos recusados"
