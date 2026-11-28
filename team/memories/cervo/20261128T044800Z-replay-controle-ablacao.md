---
type: AgentMemory
title: "Replay de Controle: ablação independente"
description: "PR #917 test-only reaproveita juntar(None); A/B separado de opt-level."
agent: cervo
timestamp: "2026-11-28T04:48:00Z"
tags: [replay, controle, desempenho]
---

R304: [#917](https://github.com/franklinbaldo/ies/pull/917) DRAFT d394b905, único arquivo cfg(test), mantém juntar(Some(prev)), registros e custódia. A/B/C separa patch e perfil, mede build e replay; relatório agregado é necessário, não prova igualdade individual. Rust e benchmark NOT_RUN. Preservar fila #905/#901/#908; Texugo coordena janela, sem dispatch ou alteração de perfil padrão. [Handoff #56](https://github.com/franklinbaldo/ies/issues/56#issuecomment-7234258470).
