---
type: AgentMemory
title: "Quatro integrações e uma causalidade ainda não demonstrada"
description: "CI 7/7 e rápido verdes nos quatro heads; #526 segue em Draft e precisa conciliar SQLite v20."
agent: texugo
timestamp: "2026-11-09T11:02:55Z"
tags: [antichurn, integracao, cassettes, cli, sqlite, causalidade]
---

A [#551](https://github.com/franklinbaldo/ies/pull/551) preserva navegação do Controle segundo a proveniência observada, com regressões antigas verdes; a [#558](https://github.com/franklinbaldo/ies/pull/558) confronta parcialmente conteúdo da cassette UnB com leitores reais e anti-IES; a [#554](https://github.com/franklinbaldo/ies/pull/554) comprova 14 redirects históricos de atribuição sem ID decimal singular e evita declarar autoria do efeito por coincidência na releitura; a [#553](https://github.com/franklinbaldo/ies/pull/553) isola ciências de dois documentos na CLI/SQLite. Quatro merges confirmados com CI oficial 7/7 e rápido verdes nos heads.

Provas diferentes: #551 e #553 são regressões sintéticas; #558 tem confronto independente parcial da cassette; #554 caracteriza POST/302 original, **não** comprova efeito ou equivalência do redirect sintético. Live NOT_RUN em todas. O BDD-271-02 da #553 segue parcial para identidade do usuário da sessão. Não executar produção, inventar subcódigos ou usar falta de cassette como veto.

Retomar [#526](https://github.com/franklinbaldo/ies/pull/526), ainda Draft com migração v18 obsoleta (v18 atribuição, v19 ciência; criação deve usar próxima versão livre, antes sugerida v20) e merge verdadeiro da main; não editar branch da Coruja. Li o [Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388): o 302 funciona como claquete, não como testemunha da mão que mudou o estado; analogia, não evidência do SEI.
