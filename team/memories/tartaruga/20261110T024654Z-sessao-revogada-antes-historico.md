---
type: AgentMemory
title: "BDD-60-05: sessão revogada antes da lista de versões"
description: "PR #630 acrescenta E2E CLI real, anti-IES HTTP/TLS e SQLite para sessão revogada depois da árvore; CI 7/7, inspeção UnB íntegra, corrida sintética."
agent: tartaruga
timestamp: "2026-11-10T02:46:54Z"
tags: [bdd60, anti-ies, navegacao, sessao, sqlite, cassette-first, cli]
---

# Retomada

**Fato:** a [PR #630](https://github.com/franklinbaldo/ies/pull/630) acrescenta a variante de BDD-60-05 em que a árvore já oferece `documento_versao_listar`, mas a sessão global no SQLite é revogada antes do GET do histórico. A CLI real atravessa TLS local e anti-IES, com e sem `--ocultar-inacessiveis`, e deve recusar HTTP 403/exit 1 sem GET de corpo, retry, fallback, documentos parciais ou POST mutador. A conferência SQLite demonstra sessão recusada e documento preservado. Não confundir com cancelamento do documento. Sem código produtivo alterado.

**Evidências:** [CI completo #42744514511](https://github.com/franklinbaldo/ies/actions/runs/42744514511) **7/7 PASS** e [rápido #42744498885](https://github.com/franklinbaldo/ies/actions/runs/42744498885) PASS no head exato `e0bcd555f18109250bf1ceaff5daa98c0d8c779f`. Não houve RED funcional: a recusa já funcionava; o CI anterior falhou apenas em dois `clippy::doc_markdown` de comentários, resolvidos sem supressões. [Inspeção UnB #42744106720](https://github.com/franklinbaldo/ies/actions/runs/42744106720) validou SHA-1/SHA-256 e as 27 interações (24 GET/3 POST), com GET do histórico. O manifesto temporário foi removido.

**Limite:** CLI ↔ anti-IES ↔ SQLite **PASS sintético**; ambos ↔ cassette para esta corrida **NOT_RUN**, porque a gravação não contém revogação temporal; treinamento/live e REA **NOT_RUN**. Hash íntegro não é confronto semântico. Nenhuma produção, credencial, novo subcódigo ou redução de TLS.

**Coordenação:** [Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6757460285) visitado e comentado; [issue #60](https://github.com/franklinbaldo/ies/issues/60) permanece aberta para lacunas e validação externa. PR #630 está Ready e sem merge; próxima rodada retoma revisão 🦅 e integração 🐘/🦡, conciliando esta memória na branch por merge verdadeiro e revalidando gates se o head mudar.
