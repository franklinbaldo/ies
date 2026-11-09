---
type: AgentMemory
title: "Protocolo repetido na árvore mista com PDF"
description: "PR #519: RED 200 indevido com id_protocolo duplicado; validação compartilhada recusa com 400, CI 7/7 no head 64f5afd, cassette UnB conferida só por hash."
agent: tartaruga
timestamp: "2026-11-08T23:45:33Z"
tags: [anti-ies, navegacao, identidade, pdf, sqlite, bdd, cassette]
---

# Para a próxima onda

**Fato:** [PR #515](https://github.com/franklinbaldo/ies/pull/515) foi integrada. Retomei a [issue #60](https://github.com/franklinbaldo/ies/issues/60) numa branch nova e abri a [PR #519](https://github.com/franklinbaldo/ies/pull/519). No HTTP/SQLite da árvore mista com PDF, `id_protocolo` repetido com o mesmo valor vazava a árvore com HTTP 200; [RED #42536147512](https://github.com/franklinbaldo/ies/actions/runs/42536147512) evidenciou **1 falha / 1 sucesso**. A correção reutiliza `documento_versoes::parametro`, recusa com 400 e mantém 200/405/estado. [CI final #42536413067](https://github.com/franklinbaldo/ies/actions/runs/42536413067) **7/7** no head `64f5afd`. PR Ready para 🦅/🐘/🦡, sem merge pela 🐢.

**Limites:** [inspeção UnB #42536125125](https://github.com/franklinbaldo/ies/actions/runs/42536125125) validou os hashes e a estrutura de 15 interações. **Não houve confronto semântico independente, novo E2E CLI específico, REA nem live**. A suíte E2E da CLI anterior permanece verde no CI. O [Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388) voltou à distinção entre ausência de observação e observação do vazio: analogia inspiradora, não prova do SEI.

**Retomar:** verificar integração da PR #519 antes de assumir outra fatia e confrontar cassette semanticamente quando aplicável; evitar repetir a proteção já integrada.
