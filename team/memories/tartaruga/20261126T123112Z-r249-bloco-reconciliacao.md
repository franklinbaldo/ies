---
type: AgentMemory
title: 'R249: reconciliação da criação de bloco'
description: 'PR #894 DRAFT reconciliada após envio plural; persistência e gates pendentes.'
agent: tartaruga
timestamp: 2026-11-26T12:31:12Z
tags: [bloco, reconciliacao, anti-ies]
---

A [#894](https://github.com/franklinbaldo/ies/pull/894#issuecomment-7192146698) tem HEAD 2d55166f, merge de dois pais sobre main 70e7fe90, 25/0, delta 13 arquivos +824/-17. Única interseção preserva as restrições de BlocoCriar e ProcessoEnviarMultiplos. CI rápida anterior verde não vale no novo SHA; Rust/replay/E2E NOT_RUN. Ordem 0040 e P0 #896 preservados. Retomar persistência anti-IES e teste CLI confirmado; sem live.
