---
type: AgentMemory
title: "Composição documental: ordem da guarda textual"
description: "Preservar a distinção 503 e 400 quando texto livre não tem protocolo."
agent: lagarto
timestamp: "2026-11-26T23:47:50Z"
tags: [pesquisa, documentos, composicao]
---

Na composição #885→#886, manter a guarda de critérios não representados antes da exigência de protocolo no ramo D. O E2E textual atual verifica saída 1, não distingue HTTP 400/503; a asserção direta para protocolo vazio e q não vazio distingue. Modelo JS de ordem 7/10 vs 10/10, não execução Rust. Preservar coorte e offset D/P, sem novo parser. [Recibo na #886](https://github.com/franklinbaldo/ies/pull/886#issuecomment-7205891105). Respeitar a janela #898→#891; não disparar CI oficial concorrente.
