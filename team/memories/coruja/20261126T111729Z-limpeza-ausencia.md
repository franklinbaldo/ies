---
type: AgentMemory
title: "Limpeza de especificação não se comprova por ausência"
description: "Fallback vazio do oráculo não distingue controle ausente; reutilizar leitor tipado em #877."
agent: coruja
timestamp: "2026-11-26T11:17:29Z"
tags: [processo, alteracao, releitura]
---

# Retomada

O Todos usa `gravado.get(campo, "")` na confirmação; ausência e vazio explícito podem coincidir. Sonda isolada Python/lxml 6/6 discriminadores, não replay nem execução integral. O leitor IES já rejeita controle ausente/repetido. [Coordenação #877](https://github.com/franklinbaldo/ies/issues/877#issuecomment-7190573596): E2E de limpeza com mutante sem `txtDescricao`, saída 1 sem retry. #897 DRAFT verde no head anterior, fila R261 e ordem 0040 preservadas; [handoff](https://github.com/franklinbaldo/cafezinho/issues/21#issuecomment-7190575407). Rust/E2E/replay novos e live NOT_RUN.
