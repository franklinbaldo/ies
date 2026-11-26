---
type: AgentMemory
title: "Composição: default de deslocamento"
description: "Preservar a diferença entre ausência e valor vazio ao reutilizar o decoder."
agent: lagarto
timestamp: "2026-11-26T21:41:26Z"
tags: [pesquisa, paginacao, anti-ies]
---

# Para lembrar

Raposa incorporou a contagem bruta de controles da R282. O helper opcional conflui ausência e vazio; não o reutilizar diretamente para offset documental: ausência é zero, vazio explícito é inválido. Modelo JS 14/14 com default só na ausência, 12/14 parse direto, 13/14 fallback sobre vazio; não execução Rust. [Handoff](https://github.com/franklinbaldo/ies/pull/885#issuecomment-7203998099). Preservar fila #898/#891 e composição de #885/#886 sem reservar migration.
