---
type: AgentMemory
title: "CI verde e memórias aditivas exigem sincronização curta"
description: "Em #487, sete jobs passaram mas a main recebeu memórias antes do merge; #493 e #491 também amadurecem enquanto a sequência SQLite v13→v14→v15 depende de integração ordenada."
agent: elefante
timestamp: "2026-11-08T19:12:36.211Z"
tags: [integracao, anti-ies, sqlite, ci, memoria]
---

# Retomar sem patinar

Fato: [#487](https://github.com/franklinbaldo/ies/pull/487) tem CI oficial 7/7 no head `218c055`, mas `main` recebeu commits aditivos de memória OKF após o último merge da branch; sincronizar por merge verdadeiro e repetir CI no SHA final antes da integração.

Fato: [#493](https://github.com/franklinbaldo/ies/pull/493) tem teste real de CLI para propriedades não publicadas e correção de fallback de identidade, com CI oficial em curso na conferência; [#492](https://github.com/franklinbaldo/ies/pull/492) tem regressão HTTP de unidade ativa ainda a sincronizar. [#489](https://github.com/franklinbaldo/ies/pull/489) só concilia v14 após #487 v13, e [#491](https://github.com/franklinbaldo/ies/pull/491) traz v15 depois da v14. [Roadmap #56](https://github.com/franklinbaldo/ies/issues/56) e [comentário #487](https://github.com/franklinbaldo/ies/pull/487#issuecomment-6725834803) atualizados nesta rodada.

Hipótese de processo: commits aditivos de memórias podem reabrir `behind>0` durante CI mesmo sem conflitos de código; concentrar integração e sincronização no fim da sessão reduz repetição dos sete jobs, **sem flexibilizar os gates**. Distinguir sempre E2E sintético, confronto independente com cassette e treinamento.

No [Cafezinho](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6725474300), a marca de pareamento da madeira evidencia coexistência, não sequência causal: inspiração ficcional, não dado do SEI.
