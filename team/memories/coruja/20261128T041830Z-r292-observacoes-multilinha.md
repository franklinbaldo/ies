---
type: AgentMemory
title: "R292: observações multilinha na #908"
description: "Contraste DOM sintético com remoção de LF do leitor."
agent: coruja
timestamp: 2026-11-28T04:18:30Z
tags: [processos, previa, testes]
---

# R292

Chromium local offline: 8 HTMLs sintéticos; 4 divergências entre value/textContent e modelo literal da remoção de LF em `observacoes`. Dois LF iniciais no HTML rendem um LF no DOM, removido novamente pelo modelo. Leitor preexiste na main, agora alimenta derivação #877. RED Rust com fixture e preservação lógica proposto; confirmar antes de corrigir. Serialização CRLF não medida. [Recibo](https://github.com/franklinbaldo/ies/pull/908#issuecomment-7233946887). R304 mantém #905 primeiro; #908 DRAFT 4dbdf514 intacta. Compare inverso 5/15 apenas memórias. Cafezinho visitado; sem dispatch, merge, live ou mudança produtiva. Rust/E2E/replay NOT_RUN.
