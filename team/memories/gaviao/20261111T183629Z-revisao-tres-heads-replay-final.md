---
type: AgentMemory
title: "Três heads novos revisados; anexação fechou CI, mas integrações defasadas"
description: "Pareceres #781, #792, #778 e atualização #777. Quatro replays originais no CI 7/7, todos com head exato; as branches continuam atrás da main."
agent: gaviao
timestamp: "2026-11-11T18:36:29Z"
tags: [revisao, cassette-first, antipatterns, ci, anti-ies, cafezinho]
---

# Retomar na próxima rodada

- [#781](https://github.com/franklinbaldo/ies/pull/781#issuecomment-6804280215) head `9f80d86`: [CI #43075678387](https://github.com/franklinbaldo/ies/actions/runs/43075678387) 7/7; `scripts/replay acervo` executou o teste nominal de troca (32 cassettes), 34 acervo + 18 cassette; `behind=2`. Bilateral anti-IES e CLI↔SQLite da troca: NOT_RUN nesta ablação de testes.
- [#792](https://github.com/franklinbaldo/ies/pull/792#issuecomment-6804280870) head `ed42c3c`: [CI #43073587405](https://github.com/franklinbaldo/ies/actions/runs/43073587405) 7/7 e acervo 34+18; `behind=4`. Base64 inválido é contraprova sintética. Sugestão não bloqueante: valor JSON nulo ainda se confunde com chave ausente em `corpo()`, caso o schema o proíba.
- [#778](https://github.com/franklinbaldo/ies/pull/778#issuecomment-6804281538) head `a0a32a8`: [CI #43071798491](https://github.com/franklinbaldo/ies/actions/runs/43071798491) 7/7 e acervo 34+18; `behind=8`. `#[source]` nas três variantes, regressão Rust; comparação externa desse defeito não capturada.
- [#777](https://github.com/franklinbaldo/ies/pull/777#issuecomment-6803140855) mesmo head `d669202`: atualizei parecer após [CI #43070401336](https://github.com/franklinbaldo/ies/actions/runs/43070401336) 7/7; replay GET bilateral nominal 10+10, 35 acervo + 18 cassette; `behind=9`. HTTP 503 depois do POST ainda sintético, live NOT_RUN. Não publiquei segundo parecer no mesmo SHA.
- Li integralmente AGENTS, ficha, cinco memórias e guia; visitei [Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388), incluindo a conversa sobre replay que imprime sem reprovar. Próximo ciclo: examinar heads novos e sincronização/CI de merge sem herdar PASS de SHA antigo. Nenhum merge, código funcional, acesso SEI, produção ou retry.
