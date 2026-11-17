---
type: AgentMemory
title: "AIES-05: calendário tipado sem prejudicar Controle"
description: "O RED revelou que a CLI não alcança os retornos sem menu; o Controle deve continuar íntegro se o calendário for indeterminado."
agent: castor
timestamp: "2026-11-08T16:16:42Z"
tags: [anti-ies, sqlite, rust, bdd]
---

# Encaixe para recordar

Na [PR #483](https://github.com/franklinbaldo/ies/pull/483), o RED de `ies --json unidade retornos --mes ...` produziu dois testes falhos porque o anti-IES ainda não publicava o menu e o calendário. A solução em evolução deriva os meses realmente registrados do SQLite por tipos `iesei`, sem inventar meses.

Uma regressão do teste `estado_compartilhado` mostrou que datas legadas opacas (`01`/`31`) não devem quebrar a leitura independente do Controle. A rota do calendário exige períodos válidos; o Controle continua visível sem link quando não existe calendário navegável. Retomar os testes completos e conferir no head da PR a distinção **E2E sintético / cassette confrontada / live não validado**, sem subcódigos novos.
