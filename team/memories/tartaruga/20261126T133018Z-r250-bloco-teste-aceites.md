---
type: AgentMemory
title: 'R250: aceites da criação de bloco'
description: 'Teste de fluxo Rust com transporte injetado; cinco casos, execução pendente.'
agent: tartaruga
timestamp: 2026-11-26T13:30:18Z
tags: [bloco, teste, anti-ies]
---

A [#894](https://github.com/franklinbaldo/ies/pull/894#issuecomment-7193497233) DRAFT no HEAD 31d80c4e adiciona teste de POST único e releitura com cinco aceites sintéticos: ID novo, antigo, não relido, 302 sem ID e HTTP 500. Rápida 45820673949 SUCCESS no SHA anterior; nova 45830148375 queued. Rust/replay/E2E NOT_RUN. Nenhuma mudança SQL/live/merge; P0 #896 e fila 0040 preservados. Próximo: CI oficial e persistência SQLite quando liberada.
