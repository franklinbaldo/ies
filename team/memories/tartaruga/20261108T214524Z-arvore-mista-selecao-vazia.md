---
type: AgentMemory
title: "Árvore mista: seleção vazia é diferente de identidade duplicada"
description: "A correção de id_documento repetido com PDF expôs a forma válida id_documento=; preservar BDD-332-03 sem fallback ambíguo."
agent: tartaruga
timestamp: "2026-11-08T21:45:24Z"
tags: [anti-ies, navegacao, documento, pdf, identidade, ci]
---

# Retomar

Fato: após a integração da [PR #493](https://github.com/franklinbaldo/ies/pull/493), abri a [PR #512](https://github.com/franklinbaldo/ies/pull/512) para recusar seleção documental repetida na **árvore mista com PDF**; o teste usa HTTP loopback e SQLite com documento cadastrado. O primeiro CI da correção revelou regressão no BDD-332-03: `id_documento=` **único e vazio** é uma forma legítima de árvore sem seleção. A função compartilhada distingue essa forma das **duas ocorrências** de `id_documento`, sem liberar fallback ambíguo. Conferir o [CI final #42526651706](https://github.com/franklinbaldo/ies/actions/runs/42526651706) no head `1dfd311` antes de revisão/integração.

Cassette-first: a [inspeção Actions #42526088513](https://github.com/franklinbaldo/ies/actions/runs/42526088513) verificou hashes de 2 cards (UnB/PEN1, 54 interações); não houve confronto semântico independente nem live. No [Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388), o tema da identificabilidade ajuda a pensar na distinção entre **ausência observada de seleção** e **identidade ambígua** — apenas analogia, não evidência sobre o SEI.
