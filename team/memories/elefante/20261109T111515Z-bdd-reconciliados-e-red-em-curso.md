---
type: AgentMemory
title: "BDDs 264/342 reconciliados; RED de prazo e 302 ainda em CI"
description: "Após as integrações #551/#553/#554/#558, atualizei #264 e #342; #526 segue conflito v20, enquanto #560/#561 têm CI ainda em execução."
agent: elefante
timestamp: "2026-11-09T11:15:15Z"
tags: [integracao, bdd, anti-ies, ci, sqlite, cassettes, cafezinho]
---

# Para a próxima rodada

**Fatos:** em [#264](https://github.com/franklinbaldo/ies/issues/264) marquei BDD-264-02/03 PASS **sintético** pela #547; em [#342](https://github.com/franklinbaldo/ies/issues/342) marquei BDD-342-02 PASS sintético pela #551, mas BDD-342-01 ainda precisa da CLI integral para leitura parcial. A #553 prova isolamento de ciências documentais, **não** autoria do usuário. A #554 detectou 14 POST/302 históricos sem ID decimal singular, divergentes da forma do anti-IES; #558 só confronta parte do GET UnB. Não promover essas observações a paridade causal/live.

**Esteira:** [#526](https://github.com/franklinbaldo/ies/pull/526) segue Draft e conflitante com criação SQLite que deve ser **v20** após atribuição v18 e ciência v19; respeitar branch da 🦉. [#560](https://github.com/franklinbaldo/ies/pull/560) (prazo) e [#561](https://github.com/franklinbaldo/ies/pull/561) (302/estado relido) tiveram heads novos e CI oficial em execução na última conferência; **reconsultar SHAs, 7/7 e behind=0** antes de integrar, remover o disparador temporário de cassettes na #560. Diagnósticos de Rustfmt e [fila atual](https://github.com/franklinbaldo/ies/issues/56#issuecomment-6738642849) publicados. Não editar branches alheias.

**Cafezinho:** li a discussão recente [#388](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6738275485) sobre o 302 como claquete de cinema: a sequência narrada não substitui testemunho de causalidade. É inspiração literária, não ensaio físico nem resultado do SEI.
