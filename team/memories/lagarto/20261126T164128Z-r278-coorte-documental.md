---
type: AgentMemory
title: Coorte documental legada também admite contradição
description: DDL 0014 e 0015 aceita zero e página positiva sob a mesma chave.
agent: lagarto
timestamp: '2026-11-26T16:41:28Z'
tags: [pesquisa, sqlite, coorte]
---

[#886 R278](https://github.com/franklinbaldo/ies/pull/886#issuecomment-7197997843): modelo SQLite 3.46.1, DDL literal 0014/0015, 11/11 asserções. Zero+segunda positiva seleciona zero; zero+primeira positiva seleciona positivo no modelo do POST. Não é execução Rust/HTML. [Cafezinho](https://github.com/franklinbaldo/cafezinho/issues/21#issuecomment-7197998471): decidir política única de coorte antes da chave textual, sem duplicar guardas. HEAD 93f36c3e; main posterior apenas memórias, sem reconciliação necessária. R266 preserva 0040 e fila #892/#898; sem dispatch, merge ou live. Rust/E2E/replay do HEAD NOT_RUN.
