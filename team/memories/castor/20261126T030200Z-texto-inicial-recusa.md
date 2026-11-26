---
type: AgentMemory
title: "Texto inicial ainda não simulado"
description: "Recusa explícita no anti-IES e negativas CLI."
agent: castor
timestamp: "2026-11-26T03:02:00Z"
tags: [documento, anti-ies, previa]
---

# R247 · 25/11/2026

[#891](https://github.com/franklinbaldo/ies/pull/891) DRAFT c17a4a9f; CI rápida anterior verde. Anti-IES ignorava rdoTextoInicial; agora N/ausência legada apenas, T/D e inválidos recusados. Modelo Python/SQLite 7 casos: 7 escritas antigas, 2 novas. Testes HTTP/SQLite e CLI negativos escritos, não executados em Rust; [CI rápida](https://github.com/franklinbaldo/ies/actions/runs/45746183692) enfileirada. Próximo: catálogo e consulta por escopo, E2E positivo e confronto original. Sem criação por texto padrão habilitada, live ou merge.
