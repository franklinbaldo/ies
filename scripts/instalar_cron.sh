#!/usr/bin/env bash
# ==============================================================================
# scripts/instalar_cron.sh — Instalador de agendamento cron para o time IES
# ==============================================================================
# Gera ou instala as entradas no crontab do Linux baseadas no AGENTS.md:
#   :05 -> 🐘 Elefante (Integração e fila)
#   :10 -> 🦊 Raposa (Implementação / UX CLI)
#   :22 -> 🦉 Coruja (Processos e reconciliação)
#   :34 -> 🐢 Tartaruga (Fronteiras Web)
#   :40 -> 🦅 Gavião (Revisão independente de PRs)
#   :46 -> 🐙 Polvo (Documentos e editor)
#   :52 -> 🦡 Texugo (Antichurn e priorização)
#   :58 -> 🦫 Castor (Qualidade Rust e DRY)
#
# Uso:
#   ./scripts/instalar_cron.sh [--print | --apply | --uninstall | --simples]
# ==============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
EXECUTOR="${SCRIPT_DIR}/rotina_agentes.sh"

MODO="${1:---print}"

gerar_linhas_completas() {
  cat <<CRON
# =================== IES AGENT ROTINAS (AGENTS.md) ===================
5  * * * * "${EXECUTOR}" elefante >/dev/null 2>&1
10 * * * * "${EXECUTOR}" raposa   >/dev/null 2>&1
22 * * * * "${EXECUTOR}" coruja   >/dev/null 2>&1
34 * * * * "${EXECUTOR}" tartaruga >/dev/null 2>&1
40 * * * * "${EXECUTOR}" gaviao   >/dev/null 2>&1
46 * * * * "${EXECUTOR}" polvo    >/dev/null 2>&1
52 * * * * "${EXECUTOR}" texugo   >/dev/null 2>&1
58 * * * * "${EXECUTOR}" castor   >/dev/null 2>&1
# =====================================================================
CRON
}

gerar_linhas_simples() {
  cat <<CRON
# =================== IES AGENT ROTINAS (SIMPLIFICADA) ================
5  * * * * "${EXECUTOR}" elefante >/dev/null 2>&1
10 * * * * "${EXECUTOR}" raposa   >/dev/null 2>&1
40 * * * * "${EXECUTOR}" gaviao   >/dev/null 2>&1
52 * * * * "${EXECUTOR}" texugo   >/dev/null 2>&1
# =====================================================================
CRON
}

case "${MODO}" in
  --print)
    echo "📋 Linhas geradas para o crontab (execute 'crontab -e' e cole):"
    echo ""
    gerar_linhas_completas
    echo ""
    echo "Dica: para instalar automaticamente no crontab do seu usuário, execute:"
    echo "  ${0} --apply"
    ;;
  --simples)
    echo "📋 Linhas para grade simplificada (4 agentes):"
    echo ""
    gerar_linhas_simples
    ;;
  --apply)
    if ! command -v crontab >/dev/null 2>&1; then
      echo "❌ Utilitário 'crontab' não encontrado neste ambiente." >&2
      echo "Copie e cole as linhas acima no cron do seu host Linux." >&2
      exit 1
    fi
    TMP_CRON="$(mktemp)"
    crontab -l 2>/dev/null | grep -v "IES AGENT ROTINAS" | grep -v "${EXECUTOR}" > "${TMP_CRON}" || true
    gerar_linhas_completas >> "${TMP_CRON}"
    crontab "${TMP_CRON}"
    rm -f "${TMP_CRON}"
    echo "✅ Entradas de agendamento do time IES instaladas no crontab com sucesso!"
    echo "Para conferir, execute: crontab -l"
    ;;
  --uninstall)
    if ! command -v crontab >/dev/null 2>&1; then
      echo "❌ Utilitário 'crontab' não encontrado." >&2
      exit 1
    fi
    TMP_CRON="$(mktemp)"
    crontab -l 2>/dev/null | grep -v "IES AGENT ROTINAS" | grep -v "${EXECUTOR}" > "${TMP_CRON}" || true
    crontab "${TMP_CRON}"
    rm -f "${TMP_CRON}"
    echo "🗑️ Entradas do IES removidas do crontab."
    ;;
  *)
    echo "Opção inválida: ${MODO}"
    echo "Uso: ${0} [--print | --apply | --uninstall | --simples]"
    exit 2
    ;;
esac
