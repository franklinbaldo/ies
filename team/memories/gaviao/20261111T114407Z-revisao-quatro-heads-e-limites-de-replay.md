---
type: AgentMemory
title: "Revisões 777, 778, 780 e 781: replay aprovado, preservar orçamento operacional"
description: "Quatro pareceres por SHA; #780 executou 24+18 testes de replay sobre 495 cards/29 ZIPs, mas o texto normativo precisa preservar POST único, sem retry e orçamento live. Outros três diffs tecnicamente favoráveis aguardam sincronização."
agent: gaviao
timestamp: "2026-11-11T11:44:07Z"
tags: [revisao, cassette-first, antipatterns, ci, seguranca-operacional, cafezinho]
---

# Ponto de retomada

- [#781, head `0862c1d`](https://github.com/franklinbaldo/ies/pull/781#issuecomment-6794894793): ablação exclusivamente de teste; dez desvios após POST mantidos, `too_many_lines` removido, [CI #43011003535](https://github.com/franklinbaldo/ies/actions/runs/43011003535) 7/7; `behind=4`.
- [#778, head `e2715eb`](https://github.com/franklinbaldo/ies/pull/778#issuecomment-6794895631): três `#[source]` existentes ligados, RED funcional anterior e [GREEN 7/7](https://github.com/franklinbaldo/ies/actions/runs/43007135119); `behind=9`. Sem E2E Web novo para a causa local.
- [#777, head `902297b`](https://github.com/franklinbaldo/ies/pull/777#issuecomment-6794896275): HTTP 503 depois de POST com E2E CLI real↔anti-IES/SQLite sintético, sem retry e erro causal; [CI #43012984356](https://github.com/franklinbaldo/ies/actions/runs/43012984356) 7/7; `behind=1`. Hash dos cards UnB/GO não prova 503 real; bilateral/live NOT_RUN.
- [#780, head `8aa282e`](https://github.com/franklinbaldo/ies/pull/780#issuecomment-6794897692): [CI #43009359853](https://github.com/franklinbaldo/ies/actions/runs/43009359853) 7/7; logs mostram 495 cards/29 ZIPs verificados, 24 replays `acervo` e 18 `cassette_` PASS. **Ajuste normativo solicitado:** não confundir “contagem exata de GET não é métrica de paridade” com dispensa de orçamento autorizado de live, proibição de retry e POST mutador único. `behind=5`. Nem todos os 495 cards têm confronto bilateral.
- Li AGENTS.md, a ficha, cinco memórias recentes, guia de cassettes, contratos e cards originais relevantes. Visitei o [Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388): a 🦊 explicou a ablação da #781 com preservação dos dez desvios, sem novo teste de paridade.

Na próxima rodada, verificar **heads novos**, correção da política na #780, sincronização das demais e eventual CI no SHA final. Sem Gherkin, commits funcionais, merge ou acesso ao SEI.
