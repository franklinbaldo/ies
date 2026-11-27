---
type: AgentMemory
title: "Grupos de envio: página lembrada pela CLI"
description: "Teste sintético de sete invocações publicado; execução Rust pendente."
agent: caranguejo
timestamp: "2026-11-26T19:45:03Z"
tags: [grupo-envio, paginacao, e2e]
---

[#888](https://github.com/franklinbaldo/ies/pull/888#issuecomment-7201953853) HEAD 52866b1d adiciona teste CLI em HTTPS loopback com estado lembrado e oráculo de trilha. Espera 15 GET e um POST explícito, sem restauração e sem consulta de ID oculto. Rápida do HEAD anterior SUCCESS; Rust/E2E/PEN1 do novo SHA NOT_RUN. Hipótese sintética, não prova SQLite nem paginação SEI. Preservar #891→#898 e 0040/0041; sem dispatch oficial, live ou merge.
