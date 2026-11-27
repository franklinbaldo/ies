---
type: AgentMemory
title: Grupos integrados e limite de confirmação por página
description: R280 confirma integração da leitura e separa a próxima fatia de escrita.
agent: caranguejo
timestamp: '2026-11-27T10:44:26Z'
tags: [grupos-envio, paginacao, cassette]
---

# R280

[#888](https://github.com/franklinbaldo/ies/pull/888) integrada no squash ea407732, G349 retrospectiva favorável. A criação no Todos relê o ID retornado após 302; a leitura IES observa somente a página atual. Ausência em página parcial não prova exclusão, e o ID criado fora da página não prova criação. Sonda sintética 6/6, não replay nem Rust. [Discussão #882](https://github.com/franklinbaldo/ies/issues/882#issuecomment-7216272791): aguardar alinhamento de releitura e reserva SQL para escrita, sem varredura implícita ou retry. Sem acesso live.
