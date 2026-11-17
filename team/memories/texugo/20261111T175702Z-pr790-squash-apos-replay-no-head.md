---
type: AgentMemory
title: "PR #790 integrada: teste vazio convertido em contraprova"
description: "Squash da ablação com CI oficial 7/7 e replay original no head exato; divergência da main era só de memórias."
agent: texugo
timestamp: "2026-11-11T17:57:02Z"
tags: [antichurn, ablacao, github-actions, cassette-e2e, integracao]
---

# Para a próxima rodada

**Fato:** depois de ler `AGENTS.md`, `team/texugo.md`, cinco memórias OKF e o [Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388), confirmei [CI oficial #43063066460](https://github.com/franklinbaldo/ies/actions/runs/43063066460) **7/7 PASS** no head `55397bc5195974b5d104ce2d40742431bac8bf50`: execução real de `scripts/replay acervo`, 495 cards, 29 ZIPs conferidos por hash, 34 testes de replay mais 18 `cassette_` PASS. Integrada [PR #790](https://github.com/franklinbaldo/ies/pull/790) por squash no [commit 2e9e43b](https://github.com/franklinbaldo/ies/commit/2e9e43b290e4331158541b4b871e3040a9692cd5); apenas um teste Rust literalmente vazio passou a verificar recusa do formulário de login sintético. Não houve mudança de produção.

**Decisão antichurn documentada na [PR](https://github.com/franklinbaldo/ies/pull/790#issuecomment-6803296497):** a main havia avançado seis commits, porém o diff entre bases demonstrava **somente seis arquivos novos de memórias de colegas**, disjuntos de toda a árvore funcional e da regressão; o HEAD validado não mudou. Por isso, squash direto sem novo ciclo de merge e replay dos mesmos bits. Não flexibiliza requisito de replay no head quando o código mudar.

**Limites:** não foi encontrada cassette da negativa de login; as comparações específicas IES↔cassette, anti-IES↔cassette, CLI↔anti-IES↔SQLite e live seguem **NOT_RUN**. O PASS geral cobre apenas leitores e rotas realmente executados no CI, não autenticação remota. Nenhum segredo, SEI de produção ou escrita externa.

**Retomar:** procurar a próxima ablação concreta nas issues #716–#723; verificar [#781](https://github.com/franklinbaldo/ies/pull/781) e [#792](https://github.com/franklinbaldo/ies/pull/792) sem mexer nas branches dos colegas, exigindo replay pertinente no head final. Evitar merges repetitivos de memórias que não alteram comportamento.
