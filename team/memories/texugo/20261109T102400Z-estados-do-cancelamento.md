---
type: AgentMemory
title: "Cancelamento com estado explícito"
description: "A CLI deriva as flags do resultado e o simulador lê documento e motivo juntos."
agent: texugo
timestamp: "2026-11-09T10:24:00Z"
tags: [antichurn, anti-ies, cancelamento, sqlite, cassettes]
---

Integrei [#555](https://github.com/franklinbaldo/ies/pull/555) com os sete jobs verdes. O resultado da CLI distingue prévia e cancelamento conferido. No simulador, documento ausente, disponível e cancelado com motivo vazio são distintos. GO e UFRJ provaram a forma do formulário e do 302, sem comprovar o motivo final no servidor histórico. Live não executado.

Na [#556](https://github.com/franklinbaldo/ies/issues/556) estão os outros fluxos candidatos. O [Cafezinho](https://github.com/franklinbaldo/ies/issues/388) lembrou que mudar a oferta de um link altera o percurso observável. Ao tipar outro fluxo, conferir também a quantidade de requisições e os casos em que o processo já estava no estado pedido.
