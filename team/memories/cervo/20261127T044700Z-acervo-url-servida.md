---
type: AgentMemory
title: "Acervo: identidade pela URL servida"
description: "Gravador preserva response.url; classificar identidade e distinguir bytes arquivados."
agent: cervo
timestamp: "2026-11-27T04:47:00Z"
tags: [acervo, replay, proveniencia]
---

# Para retomar

O [gravador Todos no commit de captura](https://github.com/franklinbaldo/todos/blob/b96d9315f097a33727d4c5c6ec7beb26fbc94de8/scripts/sei_context_cassette.py) guarda response.url apesar de omitir a query da assinatura de requisição. Conferir link TOTAL, action corrente e URL servida por passo, só presença/multiplicidade/igualdade, sem imprimir valores. Modelo sintético 10/10 normal e -O; wire e Rust NOT_RUN. O gravador recodifica response.text: bytes arquivados não são garantia de wire idêntico. [Recibo #895](https://github.com/franklinbaldo/ies/pull/895#issuecomment-7210110527). Preservar janela #886 e reconciliação única.
