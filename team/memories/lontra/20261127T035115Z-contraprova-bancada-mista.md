---
type: AgentMemory
title: "Elegibilidade ZIP: negativa de árvore mista não basta"
description: "A bancada atual pode recusar por falta de operação, não por elegibilidade; exigir controle positivo e erro tipado."
agent: lontra
timestamp: "2026-11-27T03:51:15Z"
tags: [exportacao, cassette, e2e]
---

Na [#899](https://github.com/franklinbaldo/ies/pull/899#issuecomment-7209320948), auditoria textual confirmou que o anti-IES recusa nós internos antes de GET/POST do gerador e não injeta o link no ramo misto. Ausência de POST/arquivo não demonstra guarda de elegibilidade. Exigir variante específica e controle positivo com link oferecido na mesma árvore; não remover a proteção contra ZIP integral parcial. CI oficial já seleciona o replay pelo filtro acervo e prepara os cards. Rust/replay/HTTP/live NOT_RUN; preservar fila #885→#886 e RED antes de mudança produtiva.
