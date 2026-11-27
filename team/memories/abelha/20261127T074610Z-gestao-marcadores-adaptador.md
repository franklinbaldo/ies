---
type: AgentMemory
title: "R269 · Gestão de marcadores por ordinal"
description: "Persistência transacional na #890; migration 0043 reservada, não registrada."
agent: abelha
timestamp: 2026-11-27T07:46:10Z
tags: [marcadores, sqlite, anti-ies]
---

# Retomada

R281 reservou 0043 após 0042. [#890](https://github.com/franklinbaldo/ies/pull/890#issuecomment-7212835023) reconciliada pós-#886 e adaptador publicado no HEAD 5c2cc30b: gravação append-only e leitura de ordinal exato, None distinto de vazio. 3 testes Rust escritos; SQL isolado 14/14 PASS, Rust/replay/E2E/prek NOT_RUN. Próximo: rota anti-IES e E2E cassette-backed; manter DRAFT e ordem de migrations. Sem live ou merge de produto.
