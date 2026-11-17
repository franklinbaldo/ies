---
type: AgentMemory
title: "Acompanhamento: ausência em G não implica ausência global"
description: "PR #665 corrige no-op de --deixar --grupo G quando só há H; RED 1 FAIL, GREEN 7/7, cassette UnB íntegra; falta CLI↔anti-IES e paridade externa."
agent: castor
timestamp: "2026-11-10T12:11:37Z"
tags: [ies, acompanhamento, bdd, rust, cassette, anti-ies]
---

# Para retomar

**Fato:** [PR #665](https://github.com/franklinbaldo/ies/pull/665) corrigiu BDD-348-03: pedir retirada de G enquanto há apenas H não é erro nem permissão para retirar H. O [RED #42812357025](https://github.com/franklinbaldo/ies/actions/runs/42812357025) isolou `Estrutura(NenhumAcompanhamento)` (2167 PASS/1 FAIL); o [GREEN #42812955488](https://github.com/franklinbaldo/ies/actions/runs/42812955488) validou 7/7 no SHA `3434058`. Fluxo injetado: quatro GET e nenhum POST. Nenhum subcódigo novo.

**Provas:** a [inspeção UnB #42812090695](https://github.com/franklinbaldo/ies/actions/runs/42812090695) conferiu hashes e 45 interações, **não** confrontou G/H; CLI real ↔ anti-IES ↔ SQLite deste BDD, replay bilateral e treinamento **NOT_RUN**. Não marcar BDD completo. `--deixar --confirmar` ainda fechado, BDD-348-01/02 pendentes.

No [Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6766914579) associei essa contraprova à distinção entre ausência local e global. Após a integração da PR, retomar a vertical CLI/anti-IES ou uma nova fatia de retirada por formulário observado, sem inventar URL ou afirmar captura de transição.
