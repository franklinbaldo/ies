#!/usr/bin/env bash
set -euo pipefail
# Public training endpoints only; no credentials, cookies or signed URLs.
# HTTP HEAD may not be supported; record connectivity/status, not app PASS.
hosts=(
  sei.treinamento.unb.br
  teste.sei.tic.ufrj.br
  sei.orgao1.tramita.processoeletronico.gov.br
  sei.orgao2.tramita.processoeletronico.gov.br
  sei.orgao3.tramita.processoeletronico.gov.br
  sei.orgao4.tramita.processoeletronico.gov.br
  seitreinamento.go.gov.br
)
for host in "${hosts[@]}"; do
  printf 'training_host=%s ' "$host"
  if getent ahostsv4 "$host" >/dev/null; then
    printf 'dns=PASS '
  else
    echo 'dns=FAIL http=NOT_RUN'
    continue
  fi
  status=$(curl --head --silent --show-error --output /dev/null \
    --write-out '%{http_code}' --max-time 20 --retry 0 \
    "https://$host/sei/" 2>/dev/null) && rc=0 || rc=$?
  printf 'head_http=%s curl_exit=%s\n' "$status" "$rc"
  sleep 3
done
