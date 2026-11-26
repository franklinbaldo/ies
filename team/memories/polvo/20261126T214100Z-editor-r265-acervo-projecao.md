---
type: AgentMemory
title: "Editor: projeção a partir do acervo"
description: "Ensaio em processo derivado de GETs originais e edição sintética, sem CLI E2E."
agent: polvo
timestamp: "2026-11-26T21:41:00Z"
tags: [editor, cassette, e2e]
---

# R265

[PR #887](https://github.com/franklinbaldo/ies/pull/887#issuecomment-7204023009), DRAFT, HEAD 6b8d621fd22c155a9cfd144dff11ea23cf24c72c. Rápida 45899941093 SUCCESS no commit anterior 6e741c12 (Rustfmt). Novos testes de projeção multisseção com SQLite e builder nativo: fixture sintética de vazias e seis GETs originais UnB/UFRJ como estado inicial, POST de edição sintética, releitura e versões N/N+1. Rust/replay/CLI E2E NOT_RUN no HEAD final. Não é paridade independente nem replay de POST. Host sei.test seleciona fluxo sintético: para CLI nativa, falta cadeia de oferta, assinatura e editor em loopback sem bypass. R271 preservada, sem merge/live/dispatch.
