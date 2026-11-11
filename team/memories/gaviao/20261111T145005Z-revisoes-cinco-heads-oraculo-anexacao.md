---
type: AgentMemory
title: "Cinco revisões por head e oráculo independente corrigido na anexação"
description: "Pareceres #781, #792, #790, #778 e #777. Quatro CIs oficiais 7/7 com replay original no SHA; #777 corrigiu auto-oráculo mas seu CI final ainda corria. Todas as branches atrás da main."
agent: gaviao
timestamp: "2026-11-11T14:50:05Z"
tags: [revisao, cassette-first, ci, antipatterns, anti-ies, oraculo-independente, cafezinho]
---

# Retomar

- [#781 `8ddd45e`](https://github.com/franklinbaldo/ies/pull/781#issuecomment-6799005715): [CI #43035581862](https://github.com/franklinbaldo/ies/actions/runs/43035581862) **7/7** com `scripts/replay acervo`, 495 cards/29 ZIPs; replay nominal `unidade_trocar::acervo::replay_de_unidade_trocar_no_acervo` com 32 cassettes. Leitor IES↔fonte PASS, bilateral anti-IES da troca NOT_RUN nesta fatia. `behind=4` em `main@cdfe459c`.
- [#792 `72f8985`](https://github.com/franklinbaldo/ies/pull/792#issuecomment-6799006502): [CI #43034175534](https://github.com/franklinbaldo/ies/actions/runs/43034175534) **7/7**, acervo 31 + cassette 18 testes PASS; base64 inválido não vira vazio; corrupção é sintética, não capturada. `behind=6`.
- [#790 `51c287e`](https://github.com/franklinbaldo/ies/pull/790#issuecomment-6799007215): [CI #43033847947](https://github.com/franklinbaldo/ies/actions/runs/43033847947) **7/7** e replays; teste anteriormente vazio agora rejeita login sintético. `behind=7`.
- [#778 `0577945`](https://github.com/franklinbaldo/ies/pull/778#issuecomment-6799009167): [CI #43032216017](https://github.com/franklinbaldo/ies/actions/runs/43032216017) **7/7** e replays; causa `Error::source()` preservada. Erro local não observado em HTTP. `behind=9`.
- [#777 `022554a`](https://github.com/franklinbaldo/ies/pull/777#issuecomment-6799010003): a objeção anterior de auto-oráculo foi corrigida em `src/processo_anexar/acervo.rs`: extrator dos fatos da fonte original sem depender de `ler_tela`, comparação bilateral da oferta GET e IDs via anti-IES dinâmico e SQLite semeado pelos dados brutos. [CI #43039971648](https://github.com/franklinbaldo/ies/actions/runs/43039971648) ainda **IN_PROGRESS** às 14:50Z, seis jobs PASS e replay nominal não confirmado neste head. HTTP 503 pós-POST é hipótese sintética, original/live NOT_RUN. `behind=2`.
- Li AGENTS.md e ficha completas, cinco memórias precedentes, guia e contratos/cards UnB/GO. Visitei [Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388), cuja conversa reforçou que relatar resultados não é fazê-los governar o gate. Não comentei por obrigação.

**Próximo ciclo:** conferir log nominal e conclusão do CI da #777, novos heads e reconciliações, sem dar PASS de um SHA a outro. Nenhum merge, código funcional, Gherkin, produção, segredo ou retry. Revisão só.
