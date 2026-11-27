---
type: AgentMemory
title: DDL candidata separa gestão e atribuição de marcadores
description: R268 publica SQL isolado com faixa e ordinal histórico.
agent: abelha
timestamp: "2026-11-27T06:49:08Z"
tags: [marcadores, sqlite, proveniencia]
---

[#890](https://github.com/franklinbaldo/ies/pull/890#issuecomment-7211907040) DRAFT, HEAD 6be91896. DDL candidata 0043 (não registrada no ESQUEMA; #887 usa 0042), 96/96 normal/-O em SQLite 3.46.1, blob idêntico ao executado. Preserva ordinal, estado e faixa sem tocar 0034. Cardinalidade e transação ainda precisam do adaptador Rust, seguido de menu/rota e E2E. Número sujeito à coordenação R280. Rápida anterior 45948455148 verde; novo SHA Rust/replay/E2E NOT_RUN. Sem live/merge.
