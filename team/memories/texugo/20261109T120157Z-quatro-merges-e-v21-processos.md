---
type: AgentMemory
title: "Quatro integrações e a criação causal precisa de SQLite v21"
description: "Integrei prazo, atribuição adversarial, confronto UnB e recusa de ciência; #526 deve reconciliar a criação em v21."
agent: texugo
timestamp: "2026-11-09T12:01:57Z"
tags: [antichurn, integracao, sqlite, cassettes, causalidade]
---

# Para a próxima rodada

As PRs [#560](https://github.com/franklinbaldo/ies/pull/560), [#561](https://github.com/franklinbaldo/ies/pull/561), [#566](https://github.com/franklinbaldo/ies/pull/566) e [#567](https://github.com/franklinbaldo/ies/pull/567) entraram na `main`, todas com sete jobs oficiais e CI rápido verdes nos heads. O confronto UnB na #566 é **parcial**: versões e metadados originais não provam corpos de revisões ausentes; a #567 comprova sinteticamente recusa de segunda ciência, não autoria da sessão nem live.

A [#526](https://github.com/franklinbaldo/ies/pull/526) permanece Draft/conflituosa: agora v18 é atribuição, v19 ciência, v20 prazo. A criação deve ocupar **v21** após merge verdadeiro da `main`, preservando o E2E com duas invocações da CLI e único POST. Deixei [orientação efetiva](https://github.com/franklinbaldo/ies/pull/526#issuecomment-6739547872); evitar comentá-la novamente sem fato novo. Não tocar na branch da 🦉.

No [Cafezinho](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6739255226), a analogia da discografia reforça uma distinção útil: metadados de um registro não são o corpo integral de cada revisão. Isto é inspiração para vigiar conclusões exageradas, não prova adicional.
