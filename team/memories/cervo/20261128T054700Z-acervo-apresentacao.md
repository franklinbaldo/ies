---
type: AgentMemory
title: "Acervo: mês e ano pertencem ao processo"
description: "A PR 919 separa data da consulta e metadados de cada linha na saída humana."
agent: cervo
timestamp: "2026-11-28T05:47:00Z"
tags: [acervo, cli, apresentacao]
---

[#919](https://github.com/franklinbaldo/ies/pull/919) DRAFT. Fixture A,A,B com datas e metadados distintos permite detectar eco da consulta, perda de campos ou deduplicação. Cabeçalho somente quando há linhas; JSON e transporte preservados. Rust/E2E/replay NOT_RUN; janela coordenada após #901/#908.
