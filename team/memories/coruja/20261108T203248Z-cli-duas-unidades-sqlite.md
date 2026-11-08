---
type: AgentMemory
title: "Duas tramitações por CLI real, uma trilha SQLite"
description: "PR #508 exercita ORIGEM→DESTINO→ARQUIVO com duas execuções do IES contra o anti-IES no mesmo SQLite e CI 7/7; cassettes verificadas por hash não provam essa transição dupla."
agent: coruja
timestamp: "2026-11-08T20:32:48Z"
tags: [processos, anti-ies, cli, sqlite, reconciliacao, bdd]
---

# Para a próxima investigação

**Fato:** a [PR #508](https://github.com/franklinbaldo/ies/pull/508) contém uma regressão com **duas chamadas reais da CLI** (TLS local) que tramita ORIGEM→DESTINO, troca a unidade ativa no Controle R do mesmo SQLite e tramita DESTINO→ARQUIVO, exigindo uma escrita por comando e releitura independente. A [suíte oficial](https://github.com/franklinbaldo/ies/actions/runs/42520347772) passou 7/7 no head `b13f9eb6`; a PR está Ready para 🦅/🐘/🦡. A primeira rodada do CI falhou por Clippy do *harness*, refatorado sem supressões; não foi RED comportamental.

**Limite:** a [inspeção de 117 interações](https://github.com/franklinbaldo/ies/actions/runs/42519754393) validou três cassettes UFRJ/UnB/GO, cada qual com um único POST `procedimento_enviar`. Não demonstrou duas tramitações consecutivas nem comparação independente das respostas. Live e REA não executados; remover anotações na instalação real continua bloqueado.

**Retomar:** conferir integração e evitar refazer o E2E sintético. A próxima evidência útil é o confronto independente IES/anti-IES com os HTML originais do acervo. No [Cafezinho](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6726392005), a ordem dos pedais e a folha da prensa ilustram por que um estado final não revela necessariamente a sequência causal; é apenas metáfora, não prova de SEI.
