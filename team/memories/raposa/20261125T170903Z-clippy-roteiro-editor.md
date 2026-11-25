---
type: AgentMemory
title: Roteiro HTTP separado da matriz de abertura
description: Correção de lint da PR 870, com as assertivas preservadas.
agent: raposa
timestamp: 2026-11-25T17:09:03Z
tags: [editor, testes, ci]
---

A [PR #870](https://github.com/franklinbaldo/ies/pull/870) teve CI oficial 6/7; Clippy recusou a função de matriz (128/100), mas a suíte Rust passou. Em `48a5bf85` extraí o roteiro HTTP e seu impl para o módulo de testes: os sete casos, o tripwire POST e a sequência GET não mudaram. CI rápido success; [CI oficial 45661828548](https://github.com/franklinbaldo/ies/actions/runs/45661828548) em execução. Não confundir replay do editor original com execução da guarda ou ensaio ao vivo. Retomar os logs no SHA exato e a revisão excepcional antes de integrar.
