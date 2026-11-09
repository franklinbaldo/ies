---
type: AgentMemory
title: "Confronto GO/UFRJ integrado; criação v19 e remoção CLI aguardam sincronização"
description: "A #530 trouxe 4 pares de formulários comparados com ZIPs sem provar o efeito; #531 e #526 ainda precisam reconciliação e CI final."
agent: elefante
timestamp: "2026-11-09T03:14:06Z"
tags: [integracao, cassettes, anti-ies, bdd, sqlite, causalidade]
---

# Para retomar

**Fato:** integrei [#530](https://github.com/franklinbaldo/ies/pull/530) na main via merge de dois pais [`c25cbe7`](https://github.com/franklinbaldo/ies/commit/c25cbe71483c5fc95de14670287537516762afce), 7/7 oficiais no head final. O replay explícito anterior confrontou **quatro GETs GO/UFRJ**: quatro formas IES ↔ anti-IES compatíveis, sem perda de identidade/codificação; **dois POSTs históricos não demonstram cancelamento**. O código do teste permaneceu igual no commit de retirada do workflow temporário. Nível live NOT_RUN. Registrar isso como prova restrita da *forma*, não da causalidade de escrita.

**Fato:** marquei [BDD-264-01](https://github.com/franklinbaldo/ies/issues/264) PASS sintético após #521/v18; BDD-264-02/03 seguem pendentes. A [#526](https://github.com/franklinbaldo/ies/pull/526) precisa incorporar a main, renumerar criação para **v19** e revalidar os dois comandos da CLI. A [#531](https://github.com/franklinbaldo/ies/pull/531) está em progresso e, após #530, precisa merge verdadeiro da main e CI completo novo; BDD-258-03 ainda é **parcial** (POST feito pelo teste, não por `--remover --confirmar` na CLI).

**Cafezinho:** a observação do [🐙 sobre mecanismo versus testemunha](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6731328214) explica por que a identidade visual de formulários não demonstra um POST causal. É analogia de identificabilidade, não prova SEI.

**Próximo ciclo:** consultar SHAs/CI das #531/#526 antes de integrar; não usar ausência de cassette/REA/live como veto, nem confundir Green sintético com efeito externo.
