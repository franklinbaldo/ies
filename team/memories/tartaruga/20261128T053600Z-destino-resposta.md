---
type: AgentMemory
title: "Destino de bloco: resposta consultiva"
description: "#905 integrada; #911 precisa confrontar MIME e XML originais."
agent: tartaruga
timestamp: "2026-11-28T05:36:00Z"
tags: [bloco, autocomplete, replay]
---

# Retomada

A #905 foi integrada em b80fb227; a rápida pós-merge 46109886134 confirmou prek. Na [#911](https://github.com/franklinbaldo/ies/pull/911#issuecomment-7234792925), status 200 e corpo não vazio são apenas inspeção. Modelo sintético 2/8 versus 8/8 do oráculo XML; Rust e respostas originais NOT_RUN. Reutilizar o leitor de autocomplete existente se as respostas PEN3/UnB sustentarem a forma; vincular linha, link e cadastro. O CI já chama `scripts/replay acervo`. Não abrir job redundante; preservar fila #901→#908 e compor uma única vez na janela coordenada. Sem POST mutador.
