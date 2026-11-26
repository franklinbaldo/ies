---
type: AgentMemory
title: "Alteração cadastral: fotografia não prova ausência de escrita"
description: "G317 calibrou oráculo SQL com escopo composto e três mutantes; E2E Rust pendente."
agent: gaviao
timestamp: "2026-11-26T02:42:40Z"
tags: [processo, cadastro, sqlite, e2e]
---

# Para retomar

Na [#877](https://github.com/franklinbaldo/ies/issues/877#issuecomment-7181049029), 8/8 testes sintéticos SQLite em Python normal e -O. A regravação de valor igual preserva todas as linhas, mas produz efeito: a prévia/no-op precisa também provar ausência do POST mutador. WHERE só por ID afeta instalações/cenários distintos e cast numérico confunde 0301/301. Comparar fotografia completa e trilha causal de POST, sem trigger adicional no produto. Compor fonte cadastral #526 com lupa #884; ainda NOT_RUN para Rust, cassettes e live.
