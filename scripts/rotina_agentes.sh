#!/usr/bin/env bash
# ==============================================================================
# scripts/rotina_agentes.sh — Executor de agentes do IES via Antigravity CLI (agy)
# ==============================================================================
# Executa um agente em modo headless ('agy -p') com:
# - Bloqueio via flock por agente (evita sobreposição da mesma rotina)
# - Detecção automática do executável 'agy'
# - Timeout de proteção (--print-timeout 50m)
# - Logs organizados com carimbo de data/hora
#
# Uso:
#   ./scripts/rotina_agentes.sh [elefante|raposa|coruja|tartaruga|gaviao|polvo|texugo|castor|geral|auto]
# ==============================================================================

set -euo pipefail
umask 077  # Logs e locks privados desde a primeira criação.

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"

# Diretório privado de logs (inclusive se já existia de uma versão anterior).
LOG_DIR="${IES_LOG_DIR:-/tmp/agy-ies/logs}"
mkdir -p -m 700 "${LOG_DIR}"
chmod 700 "${LOG_DIR}"

# Localização do binário agy
AGY_BIN="${AGY_PATH:-$(command -v agy || true)}"
if [[ -z "${AGY_BIN}" ]]; then
  for cand in "/usr/local/bin/agy" "/opt/antigravity/bin/agy" "${HOME}/.local/bin/agy" "${HOME}/.antigravity/bin/agy"; do
    if [[ -x "${cand}" ]]; then
      AGY_BIN="${cand}"
      break
    fi
  done
fi

if [[ -z "${AGY_BIN}" ]]; then
  echo "❌ Erro: executável 'agy' do Antigravity não encontrado no PATH nem nos diretórios padrão." >&2
  echo "Defina a variável AGY_PATH apontando para o binário do Antigravity CLI." >&2
  exit 1
fi

# Determinar qual agente rodar
ALVO="${1:-auto}"

if [[ "${ALVO}" == "auto" ]]; then
  MINUTO="$(date +%-M)"
  case "${MINUTO}" in
    5|05)   ALVO="elefante" ;;
    10)     ALVO="raposa" ;;
    22)     ALVO="coruja" ;;
    34)     ALVO="tartaruga" ;;
    40)     ALVO="gaviao" ;;
    46)     ALVO="polvo" ;;
    52)     ALVO="texugo" ;;
    58)     ALVO="castor" ;;
    *)
      # Minuto intermediário: pega o agente anterior mais recente ou geral
      if (( MINUTO >= 5 && MINUTO < 10 )); then
        ALVO="elefante"
      elif (( MINUTO >= 10 && MINUTO < 22 )); then
        ALVO="raposa"
      elif (( MINUTO >= 22 && MINUTO < 34 )); then
        ALVO="coruja"
      elif (( MINUTO >= 34 && MINUTO < 40 )); then
        ALVO="tartaruga"
      elif (( MINUTO >= 40 && MINUTO < 46 )); then
        ALVO="gaviao"
      elif (( MINUTO >= 46 && MINUTO < 52 )); then
        ALVO="polvo"
      elif (( MINUTO >= 52 && MINUTO < 58 )); then
        ALVO="texugo"
      else
        ALVO="castor"
      fi
      ;;
  esac
fi

# Mapeamento do prompt conforme AGENTS.md e fichas em team/
case "${ALVO}" in
  elefante)
    EMOJI="🐘"
    PAPEL="Integração e fila"
    PROMPT="Você é o 🐘 Elefante. Leia AGENTS.md e team/elefante.md. Inspecione PRs abertas com CI oficial completo 7/7 verde no head exato, sincronizadas com a main, e integre as fatias aprovadas via merge commit. Mantenha a fila aberta em dia. Participe do Cafezinho diário do dia quando couber."
    ;;
  raposa)
    EMOJI="🦊"
    PAPEL="UX CLI e implementação"
    PROMPT="Você é a 🦊 Raposa. Leia AGENTS.md e team/raposa.md. Puxe a próxima issue livre de maior prioridade na fila aberta, avance E2E da CLI com cassettes existentes, valide gates locais e abra PR."
    ;;
  coruja)
    EMOJI="🦉"
    PAPEL="Processos e reconciliação"
    PROMPT="Você é a 🦉 Coruja. Leia AGENTS.md e team/coruja.md. Puxe a próxima issue livre de maior prioridade na fila aberta, avance E2E de processos com cassettes existentes, valide gates e abra PR."
    ;;
  tartaruga)
    EMOJI="🐢"
    PAPEL="Fronteiras Web e navegação"
    PROMPT="Você é a 🐢 Tartaruga. Leia AGENTS.md e team/tartaruga.md. Puxe a próxima issue livre de maior prioridade na fila aberta, avance o próximo E2E guiado por cassettes, valide gates e abra PR."
    ;;
  gaviao)
    EMOJI="🦅"
    PAPEL="Revisão independente"
    PROMPT="Você é o 🦅 Gavião. Leia AGENTS.md e team/gaviao.md. Faça revisão técnica independente de todas as PRs abertas na esteira, verifique escopo, E2E cassette-first e emita parecer técnico nas PRs."
    ;;
  polvo)
    EMOJI="🐙"
    PAPEL="Documentos e editor"
    PROMPT="Você é o 🐙 Polvo. Leia AGENTS.md e team/polvo.md. Puxe a próxima issue livre de maior prioridade na fila aberta, avance E2E de documentos com cassettes existentes, valide gates e abra PR."
    ;;
  texugo)
    EMOJI="🦡"
    PAPEL="Antichurn e priorização"
    PROMPT="Você é o 🦡 Texugo. Leia AGENTS.md e team/texugo.md. Atue no desempate de PRs, na classificação de prioridades 0-100 da fila aberta, desbloqueio de testes e avanço pragmático da capacidade. Concluída a fatia, participe do Cafezinho diário."
    ;;
  castor)
    EMOJI="🦫"
    PAPEL="Qualidade Rust e DRY"
    PROMPT="Você é o 🦫 Castor. Leia AGENTS.md e team/castor.md. Puxe a próxima issue livre de maior prioridade na fila aberta, avance o próximo E2E guiado por cassettes, garanta consistência e abra PR."
    ;;
  geral)
    EMOJI="🤖"
    PAPEL="Rotina geral"
    PROMPT="Leia AGENTS.md. Verifique as issues e PRs abertas do repositório, escolha a tarefa prioritária na fila aberta e avance a implementação seguindo E2E cassette-first e os limites do projeto."
    ;;
  *)
    echo "Agente desconhecido: ${ALVO}" >&2
    echo "Opções válidas: elefante, raposa, coruja, tartaruga, gaviao, polvo, texugo, castor, geral, auto." >&2
    exit 2
    ;;
esac

# Mesmo agente nunca executa em paralelo; os outros não são bloqueados.
# IES_LOCK_FILE permite trava global apenas por escolha explícita do operador.
LOCK_FILE="${IES_LOCK_FILE:-${LOG_DIR}/.${ALVO}.lock}"
TIMESTAMP="$(date +%Y%m%d_%H%M%S)"
LOG_FILE="${LOG_DIR}/${TIMESTAMP}_${ALVO}.log"
touch "${LOG_FILE}"
chmod 600 "${LOG_FILE}"

echo "[$(date '+%Y-%m-%d %H:%M:%S')] ${EMOJI} Iniciando rotina ${ALVO} (${PAPEL})..."
echo "[$(date '+%Y-%m-%d %H:%M:%S')] Repositório: ${REPO_DIR}"
echo "[$(date '+%Y-%m-%d %H:%M:%S')] Log: ${LOG_FILE}"

# Execução protegida por flock por avatar: não perde as demais rodadas.
exec 200>"${LOCK_FILE}"
if ! flock -n 200; then
  echo "⚠️  [$(date '+%Y-%m-%d %H:%M:%S')] Rodada ${ALVO} pulada: o mesmo agente já está em execução (lock em ${LOCK_FILE})." | tee -a "${LOG_FILE}"
  exit 0
fi

# Disparar agy em modo headless no diretório do repositório
cd "${REPO_DIR}"

set +e
"${AGY_BIN}" -p "${PROMPT}" --print-timeout 50m >> "${LOG_FILE}" 2>&1
STATUS=$?
set -e

if [[ ${STATUS} -eq 0 ]]; then
  echo "✅ [$(date '+%Y-%m-%d %H:%M:%S')] ${EMOJI} Rotina ${ALVO} concluída com sucesso."
else
  echo "⚠️  [$(date '+%Y-%m-%d %H:%M:%S')] ${EMOJI} Rotina ${ALVO} encerrou com código de saída ${STATUS}. Verifique ${LOG_FILE}."
fi

exit ${STATUS}
