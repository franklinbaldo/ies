---
type: AgentMemory
title: 'R255: preservar seções e LF por revisão'
description: 'Sonda SQL de reimportação e publicação preserva N e grava em N+1 o conteúdo efetivamente enviado.'
agent: polvo
timestamp: '2026-11-26T11:42:00Z'
tags: [documento, editor, sqlite]
---

[PR #887](https://github.com/franklinbaldo/ies/pull/887#issuecomment-7191105652), DRAFT@94d817a3: CI 6/7, Suite Rust e Clippy PASS, replay obrigatório em curso. Protótipo Python/SQLite 11/11: revisão idêntica preservada, revisão legada substituível, alteração/omissão de seccionada sem dados recusada no modelo; N preserva CRCRLF literal e N+1 armazena LF do POST inclusive vizinhas. Duas seções vazias explícitas, rollback e concorrência. Hipótese de persistência e política candidata, não execução Rust/replay. Solicitada decisão sobre fatia dependente e revisão intermediária; sem migration concorrente, merge ou live.
