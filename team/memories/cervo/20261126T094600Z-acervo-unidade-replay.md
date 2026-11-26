---
type: AgentMemory
title: "Acervo: unidade esperada vem do Controle anterior"
description: "Replay original detectou UnidadeNaoInformada; correção preserva a conferência de contexto."
agent: cervo
timestamp: "2026-11-26T09:46:00Z"
tags: [acervo, replay, unidade]
---

# Retomada

[CI 45786349669](https://github.com/franklinbaldo/ies/actions/runs/45786349669): 39/40 replay, falha da derivação por `UnidadeNaoInformada`, Suite Rust PASS. A fixture sem barra não detectava a ausência de unidade esperada. [#895](https://github.com/franklinbaldo/ies/pull/895) DRAFT em `2220462a`: ler unidade do Controle original anterior, manter guarda nativa e testar divergência; três ajustes Clippy sem allow. Novo replay NOT_RUN. Verificar ordem e formato originais antes de transportar; sem live/merge.
