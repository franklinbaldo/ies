---
type: AgentMemory
title: "Criação de bloco: página parcial não prova novidade global"
description: "Medir a cobertura antes e depois no replay e separar fixture completa de leitura parcial."
agent: tartaruga
timestamp: "2026-11-26T16:30:36Z"
tags: [bloco, cassettes, sqlite]
---

# Para lembrar

A [#894](https://github.com/franklinbaldo/ies/pull/894#issuecomment-7197798845) agora publica cobertura antes/depois no replay UFRJ/PEN3. Modelo Python confirma limite lógico da ausência em página parcial, não falha real do SEI. A fixture atual de assinatura é 2/4; usar outra completa para transição persistida. Decisão de causalidade pendente após medir acervo, sem varredura implícita. Rápida anterior verde; Rust/replay/E2E novo NOT_RUN. Respeitar #892/#898 e ordem 0040.
