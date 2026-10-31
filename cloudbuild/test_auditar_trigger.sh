#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
cat > "$tmp/valido.json" <<'JSON'
{
  "name": "ies-pr-ci",
  "id": "ecaec137-806f-428a-8b19-e3e5b9387dcc",
  "resourceName": "projects/virtual-computer-fsb/locations/us-central1/triggers/ecaec137-806f-428a-8b19-e3e5b9387dcc",
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
aprovado="$(jq -cS '.build' "$tmp/valido.json" | sha256sum | cut -d' ' -f1)"
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
jq '.sourceToBuild = {"ref":"refs/heads/main","repoType":"GITHUB"}' "$tmp/valido.json" > "$tmp/outra-fonte.json"
recusar "$tmp/outra-fonte.json" "$aprovado"
jq '.filter = "false"' "$tmp/valido.json" > "$tmp/filtro-cel.json"
recusar "$tmp/filtro-cel.json" "$aprovado"
jq '.build.source = {"repoSource":{"branchName":"main"}}' "$tmp/valido.json" > "$tmp/build-source.json"
recusar "$tmp/build-source.json" "$aprovado"
# A impressão digital deve cobrir todo o build, não apenas quatro campos.
# Artefatos não auditados podem exfiltrar saída da compilação ou alterar o
# efeito do job mesmo que os steps permaneçam idênticos.
jq '.build.artifacts = {"objects":{"location":"gs://outro-bucket/builds/","paths":["**"]}}' "$tmp/valido.json" > "$tmp/artefatos-nao-aprovados.json"
recusar "$tmp/artefatos-nao-aprovados.json" "$aprovado"
jq '.build.substitutions = {"_MODO":"ignorar-validacao"}' "$tmp/valido.json" > "$tmp/substituicoes-nao-aprovadas.json"
recusar "$tmp/substituicoes-nao-aprovadas.json" "$aprovado"
# Dois triggers podem ter o mesmo nome/repositório/build sem serem o gate
# efetivamente exigido pelo GitHub. Exigir a identidade imutável da instância.
jq '.id = "00000000-0000-0000-0000-000000000000"' "$tmp/valido.json" > "$tmp/trigger-impersonado.json"
recusar "$tmp/trigger-impersonado.json" "$aprovado"
jq '.resourceName = "projects/virtual-computer-fsb/locations/us-central1/triggers/00000000-0000-0000-0000-000000000000"' "$tmp/valido.json" > "$tmp/outra-identidade.json"
recusar "$tmp/outra-identidade.json" "$aprovado"
echo "PASS: fonte inline, identidade do trigger e build integral aprovados; dezessete cenários negativos recusados"
