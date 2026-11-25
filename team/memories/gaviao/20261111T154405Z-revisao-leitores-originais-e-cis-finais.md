---
type: AgentMemory
title: "Replays de bloco, marcar e comentar e gates por SHA"
description: "Cinco pareceres novos (#794, #781, #792, #790, #778), com CI 7/7 comprovado; #777 atualizada após conclusão. A #794 ainda não confronta bilateralmente as novas formas com anti-IES."
agent: gaviao
timestamp: "2026-11-11T15:44:05Z"
tags: [revisao, cassette-first, antipatterns, ci, leitores, anti-ies, cafezinho]
---

# Retomar

- [#794](https://github.com/franklinbaldo/ies/pull/794#issuecomment-6800252992), head `4fe5912`: [CI #43046438115](https://github.com/franklinbaldo/ies/actions/runs/43046438115) 7/7, comando `scripts/replay acervo` executou nominalmente leitores de bloco (17 cassettes), marcar (18) e comentar (8); 34 acervo + 18 cassette PASS. Os cards GO e hashes foram consultados. Ajustes pedidos: atualizar contrato de comentários para visão A observada (sem afirmar semântica documental não provada), anti-IES do bloco ainda não emite vazio/singular, bilateral e CLI+SQLite dessas novas formas NOT_RUN. Branch divergente da main.
- [#781](https://github.com/franklinbaldo/ies/pull/781#issuecomment-6800253758) head `0df361e`: após parecer, [CI #43047004516](https://github.com/franklinbaldo/ies/actions/runs/43047004516) 7/7 executou replay nominal de troca, 32 cassettes, 31 acervo + 18 cassette PASS. Único commit atrás da main na comparação; bilateral de troca NOT_RUN nesta ablação de teste.
- [#792](https://github.com/franklinbaldo/ies/pull/792#issuecomment-6800254546) head `d9baf9f`: [CI #43043699375](https://github.com/franklinbaldo/ies/actions/runs/43043699375) 7/7, preserva erro de base64 corrompido no harness; corrupção sintética.
- [#790](https://github.com/franklinbaldo/ies/pull/790#issuecomment-6800255347) head `2b0f095`: [CI #43042857304](https://github.com/franklinbaldo/ies/actions/runs/43042857304) 7/7, substitui teste vazio por negativa Rust de login sintético.
- [#778](https://github.com/franklinbaldo/ies/pull/778#issuecomment-6800256102) head `06cca41`: [CI #43042028615](https://github.com/franklinbaldo/ies/actions/runs/43042028615) 7/7, três causas existentes preservadas com `#[source]`.
- [#777](https://github.com/franklinbaldo/ies/pull/777#issuecomment-6799010003) head `022554a`: atualizei parecer anterior após [CI #43039971648](https://github.com/franklinbaldo/ies/actions/runs/43039971648) 7/7, 32 acervo (inclui confronto GET de anexação) + 18 cassette; POST 503 permanece hipótese sintética, live NOT_RUN. Branch defasada.
- Consultei AGENTS.md, ficha, cinco memórias OKF recentes, guia vigente e [Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388). Próximo ciclo: revisar apenas heads novos, conferir sincronizações e replays no SHA final; nunca converter “7/7 geral” em paridade não testada. Sem código funcional, merge, produção, retry ou segredos.
