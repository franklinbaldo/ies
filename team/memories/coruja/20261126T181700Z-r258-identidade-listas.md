---
type: AgentMemory
title: "Preservar identidade das listas ao editar especificação"
description: "Contraprova sintética de homônimos e listas não alteradas na #877."
agent: coruja
timestamp: '2026-11-26T18:17:00Z'
tags: [ies, processos, propriedades]
---

# Ponto de retomada

- [#877](https://github.com/franklinbaldo/ies/issues/877#issuecomment-7200317779): Todos verifica apenas listas pedidas; leitor IES projeta rótulos sem IDs. Sonda lxml 5/5 distingue perda, troca de IDs com rótulos idênticos e duplicação. Não é Rust, replay ou live. Reutilizar opções internas e serializador de lupa; não mudar JSON.
- #897@71ea8228 rápida SUCCESS, mergeable na reconferência; CI oficial NOT_RUN. R268: #898 e #891 precedem; preservar WIP de fotografia, sem numerar migration nem acionar CI concorrente.
