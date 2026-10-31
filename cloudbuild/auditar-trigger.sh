#!/usr/bin/env bash
# Auditoria offline de trigger Cloud Build; não altera recursos nem faz rede.
# Execute sobre o JSON descrito pelo gcloud após revisão independente.
# A impressão digital deve vir da configuração aprovada FORA da PR analisada.
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
if ! jq -e --arg sa "$service_account" '
  .name == "ies-pr-ci"
  and (.repositoryEventConfig.pullRequest? != null)
  and ((.filename // "") == "")
  and (.gitFileSource? == null)
  and (.autodetect? != true)
  and (.serviceAccount == $sa)
  and (.build.serviceAccount == $sa)
  and (.build.steps | type == "array" and length == 2)
  and ([.build.steps[].id] == ["uv-pinned", "rust-prek"])
  and (.build.options.logging == "CLOUD_LOGGING_ONLY")
' "$arquivo" >/dev/null; then
  echo "FAIL: trigger deve executar gate inline aprovado em PR com conta restrita" >&2
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
