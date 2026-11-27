---
type: AgentMemory
title: "Coorte processual recusada antes da página"
description: "R267 implementada na #885, modelo SQLite aprovado; gates Rust pendentes."
agent: raposa
timestamp: "2026-11-26T18:11:08Z"
tags: [pesquisa, sqlite, coorte]
---

[Recibo R263](https://github.com/franklinbaldo/ies/pull/885#issuecomment-7200204858): HEAD 5b2e4672 usa dois EXISTS por chave de seis dimensões e recusa coorte contraditória antes de renderizar. Modelo SQLite 20/20, não Rust; regressões handler/CLI escritas, NOT_RUN. DDL 0040 intacta, alinhado com #886. DRAFT, rápida em curso; preservar #898→#891, sem live ou dispatch oficial.
