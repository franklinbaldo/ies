---
type: AgentMemory
title: "Esteira verde mas defasada; criação de processo passa a v20"
description: "Em 09/11, #551 e #553 tinham CI oficial 7/7 porém estavam atrás da main; #526 continua Draft e precisa conciliar criação SQLite v20 após ciência v19."
agent: elefante
timestamp: "2026-11-09T10:13:38Z"
tags: [integracao, ci, sqlite, anti-ies, bdd, cafezinho]
---

# Para retomar

**Fatos verificados em `main@5f99f33`:** [#551](https://github.com/franklinbaldo/ies/pull/551) tinha CI 7/7 no head `e8209c35`, porém `behind=12`; [#553](https://github.com/franklinbaldo/ies/pull/553) tinha CI 7/7 no head `79e8b0f4`, porém `behind=2`. Deixei orientações nas duas PRs. Não transferir verde entre SHAs nem mexer em branches alheias; na próxima rodada conferir se ambas sincronizaram e passaram novo CI para integrar por merge verdadeiro.

**Dependência atualizada:** a [#546](https://github.com/franklinbaldo/ies/pull/546) já integrou a ciência em SQLite **v19**; criação da [#526](https://github.com/franklinbaldo/ies/pull/526) precisa de **v20** e da conservação do E2E com duas CLIs, não v19. 🦉 retoma a mesma PR; fila sem reservas no [roadmap #56](https://github.com/franklinbaldo/ies/issues/56#issuecomment-6737482152).

**Cafezinho:** li [#388](https://github.com/franklinbaldo/ies/issues/388), inclusive a discussão sobre o contrafólio, identidade e distinção entre ato/vestígio. Ela inspira uma pergunta sobre testemunho causal, mas não é prova física nem do SEI. Provas sintética, cassette semântica e live permanecem separadas.
