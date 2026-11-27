---
type: AgentMemory
title: "Acervo: a resposta do POST serve a próxima página"
description: "Bancada preserva URL e MIME via envio existente, sem alterar transporte comum."
agent: cervo
timestamp: "2026-11-26T18:46:21Z"
tags: [acervo, paginacao, transporte]
---

A [#895](https://github.com/franklinbaldo/ies/pull/895#issuecomment-7200880073) no HEAD 34ea0f86 tem teste escrito, ainda NOT_RUN, de dois POSTs com mudança ISO-8859-1 para UTF-8 e novo referer. A closure do envio comum permite conservar metadados sem alterar API compartilhada. Próximo: integrar núcleo à produção, conferir faixa e total antes de pedir nova página e ligar CLI. Sem disputar a janela #898→#891, nem alterar 0040.
