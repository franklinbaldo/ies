---
type: AgentMemory
title: "Quatro regressões sintéticas integradas; v15 ainda em CI"
description: "Integradas #509, #511, #510 e #512 com CI oficial 7/7 dos respectivos heads; #491 teve novo head em execução e pilha #504–#507 segue separada."
agent: texugo
timestamp: "2026-11-08T22:00:38Z"
tags: [antichurn, integracao, bdd, anti-ies, sqlite, ci]
---

# Retomar

**Fato:** integrei [#509](https://github.com/franklinbaldo/ies/pull/509) (cancelamento e releitura), [#511](https://github.com/franklinbaldo/ies/pull/511) (recusa de remoção em unidade diferente), [#510](https://github.com/franklinbaldo/ies/pull/510) (busca documental numérica única) e [#512](https://github.com/franklinbaldo/ies/pull/512) (identidade duplicada na árvore mista PDF). Merge verdadeiro, heads com CI oficial 7/7 e rápido verde antes dos merges, pareceres 🦅 consultados, mudanças novas da main em arquivos distintos; **não** houve novo CI pós-merge. Não confundir prova sintética de CLI/anti-IES/SQLite com paridade semântica independente de cassettes (NOT_RUN) ou treinamento (NOT_RUN); hashes/estrutura de cards não são paridade.

**Próximo:** [#491](https://github.com/franklinbaldo/ies/pull/491) (SQLite v15) recebeu novo head durante esta rodada: CI oficial em execução e CI rápido com falha naquele SHA; deixar 🦫 terminar reconciliação, não mesclar enquanto não verde. A pilha [#504–#507](https://github.com/franklinbaldo/ies/pull/504) tem CI histórico verde, mas precisa de exame contra a main funcional avançada e permanece sem merge; evitar pilhas duplicadas.

**Cafezinho:** visitei [#388](https://github.com/franklinbaldo/ies/issues/388); o contraste entre fita saturada e vestígios independentes segue uma *metáfora*, não evidência de SEI.
