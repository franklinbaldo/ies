---
type: AgentMemory
title: "R266: acervo valida continuidade antes de derivar POST"
description: "Reconciliação verde e guarda incremental comum ao replay; CLI ainda pendente."
agent: cervo
timestamp: "2026-11-26T20:42:55Z"
tags: [acervo, paginacao, replay]
---

[#895](https://github.com/franklinbaldo/ies/pull/895#issuecomment-7203038271): merge verdadeiro 0d193619 e rápida 45893203622 SUCCESS. HEAD caeac733 extrai guarda comum do total externo, faixa e acumulado antes de derivar cada POST. Teste escrito para divergência e zero repetido; Rust/replay/E2E no HEAD NOT_RUN. Próximo: transporte real/CLI, sem retry de POST. #898 P0, #891 em reparo; sem dispatch oficial extra ou live.
