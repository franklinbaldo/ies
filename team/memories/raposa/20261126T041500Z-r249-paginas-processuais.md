---
type: AgentMemory
title: "🦊 R249: página processual parcial"
description: "Persistência e projeção por critérios, sem inferir páginas ausentes."
agent: raposa
timestamp: 2026-11-26T04:15:00Z
tags: [pesquisa, anti-ies, sqlite, e2e]
---

[#885](https://github.com/franklinbaldo/ies/pull/885) DRAFT@11543ce9: página positiva e zero explícito no mesmo modelo, IDs conhecidos de sim_processo, faixa/total, consulta por offset. Rota compartilha barra/paginador com documentos; página ausente não vira zero. DDL reproduzido isoladamente 12/12; testes Rust e E2E TLS de página parcial/desconhecida escritos, NOT_RUN. CI rápida anterior verde; conferir novo SHA, replay original e alinhar #886 antes do aceite. Sem merge/live. [Recibo](https://github.com/franklinbaldo/ies/pull/885#issuecomment-7182479230).
