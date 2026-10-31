#!/usr/bin/env bash
# Auditoria offline de trigger Cloud Build; não altera recursos nem faz rede.
# Execute sobre o JSON descrito pelo gcloud após revisão independente.
# A impressão digital deve vir da configuração aprovada FORA da PR analisada.
# Contrato: além do gate imutável, a fonte deve ser o repositório IES autorizado
# e o evento deve contemplar PRs da main, sem filtros silenciosos de arquivos.
set -euo pipefail

if [ "$#" -ne 2 ]; then
  echo "uso: auditar-trigger.sh TRIGGER.json SHA256_APROVADO_DO_BUILD" >&2
  exit 2
fi
arquivo="$1"
aprovado="$2"
if ! [[ "$aprovado" =~ ^[0-9a-f]{64}$ ]]; then
  echo "SHA-256 aprovado deve ter 64 dígitos hexadecimais minúsculos" >&2
  exit 2
fi
if ! [ -f "$arquivo" ] || ! command -v jq >/dev/null || ! command -v sha256sum >/dev/null; then
  echo "JSON do trigger ou ferramentas jq/sha256sum indisponíveis" >&2
  exit 2
fi

service_account="projects/virtual-computer-fsb/serviceAccounts/ies-ci@virtual-computer-fsb.iam.gserviceaccount.com"
repositorio="projects/virtual-computer-fsb/locations/us-central1/connections/judicial-github/repositories/ies"
if ! jq -e --arg sa "$service_account" --arg repositorio "$repositorio" '
  .name == "ies-pr-ci"
  and (.repositoryEventConfig.repository == $repositorio)
  and (.repositoryEventConfig.repositoryType == "GITHUB")
  and (.repositoryEventConfig.pullRequest.branch == ".*")
  and (.repositoryEventConfig.pullRequest.commentControl == "COMMENTS_ENABLED_FOR_EXTERNAL_CONTRIBUTORS_ONLY")
  and ((.includedFiles // []) == [])
  and ((.ignoredFiles // []) == [])
  and ((.substitutions // {}) == {})
  and ((.filter // "") == "")
  and (.sourceToBuild? == null)
  and (.triggerTemplate? == null)
  and (.build.source? == null)
  and (.webhookConfig? == null)
  and (.disabled != true)
  and ((.filename // "") == "")
  and (.gitFileSource? == null)
  and (.autodetect? != true)
  and (.serviceAccount == $sa)
  and (.build.serviceAccount == $sa)
  and (.build.steps | type == "array" and length == 2)
  and ([.build.steps[].id] == ["uv-pinned", "rust-prek"])
  and (.build.options.logging == "CLOUD_LOGGING_ONLY")
' "$arquivo" >/dev/null; then
  echo "FAIL: trigger de PR deve conferir repositório, evento, filtros, gate inline e conta autorizada" >&2
  exit 1
fi

# Normalização estável: exclui somente metadados atribuídos pelo Cloud Build,
# mas inclui TODOS os comandos, imagens, volumes, argumentos, SA e timeout.
observado="$(jq -cS '.build | {steps, options, timeout, serviceAccount}' "$arquivo" | sha256sum | cut -d' ' -f1)"
if [ "$observado" != "$aprovado" ]; then
  echo "FAIL: gate em execução não corresponde ao SHA-256 aprovado" >&2
  exit 1
fi
echo "PASS: trigger de PR inline e gate completo correspondem ao plano aprovado"
