#!/usr/bin/env bash
# ==============================================================================
# scripts/daemon_agentes.sh — Daemon em loop contínuo para o time IES
# ==============================================================================
# Alternativa ao cron do sistema: roda em background (loop infinito),
# calculando o tempo até o próximo minuto de agenda (:05, :10, :22, :34, :40, :46, :52, :58)
# e disparando o respectivo agente com flock e logs.
#
# Uso:
#   ./scripts/daemon_agentes.sh [--simples | --completo]
# ==============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
EXECUTOR="${SCRIPT_DIR}/rotina_agentes.sh"

MODO="${1:---completo}"

if [[ "${MODO}" == "--simples" ]]; then
  MINUTOS_ALVO=(5 10 40 52)
else
  MINUTOS_ALVO=(5 10 22 34 40 46 52 58)
fi

echo "🚀 [$(date '+%Y-%m-%d %H:%M:%S')] Daemon do time IES iniciado em modo ${MODO}."
echo "Agenda nos minutos: ${MINUTOS_ALVO[*]}"

while true; do
  MINUTO_ATUAL="$(date +%-M)"
  SEGUNDO_ATUAL="$(date +%-S)"

  # Encontrar o próximo minuto agendado
  PROXIMO_MINUTO=""
  for m in "${MINUTOS_ALVO[@]}"; do
    if (( m > MINUTO_ATUAL )); then
      PROXIMO_MINUTO="${m}"
      break
    fi
  done

  if [[ -n "${PROXIMO_MINUTO}" ]]; then
    MINUTOS_ESPERA=$(( PROXIMO_MINUTO - MINUTO_ATUAL - 1 ))
    SEGUNDOS_ESPERA=$(( MINUTOS_ESPERA * 60 + (60 - SEGUNDO_ATUAL) ))
  else
    # Próximo minuto é no próximo ciclo de hora (o primeiro da lista)
    PRIMEIRO="${MINUTOS_ALVO[0]}"
    MINUTOS_RESTANTES=$(( 60 - MINUTO_ATUAL - 1 ))
    SEGUNDOS_ESPERA=$(( MINUTOS_RESTANTES * 60 + (60 - SEGUNDO_ATUAL) + PRIMEIRO * 60 ))
  fi

  echo "⏳ Próximo disparo em ${SEGUNDOS_ESPERA}s (às $(date -d "+${SEGUNDOS_ESPERA} seconds" '+%H:%M:%S'))..."
  sleep "${SEGUNDOS_ESPERA}"

  # Executar rotina com auto-seleção pelo minuto atual
  "${EXECUTOR}" auto || true
done
