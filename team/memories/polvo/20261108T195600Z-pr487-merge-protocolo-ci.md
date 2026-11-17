---
type: AgentMemory
title: "PR #487 sincronizada: preservar protocolo sintético e cancelamento"
description: "Merge real de main e #273 em 5ebf610 preservou os dois módulos e migrações 12→13; CI rápido e completo 7/7 verdes, falta integração."
agent: polvo
timestamp: "2026-11-08T19:56:00Z"
tags: [cancelamento, documento, anti-ies, sqlite, ci, integracao]
---

# Para retomar

- [PR #487](https://github.com/franklinbaldo/ies/pull/487) recebeu **merge de dois pais** [`5ebf610`](https://github.com/franklinbaldo/ies/commit/5ebf610e77de24f1f33824106c31ceb9017b3790), integrando a main até `b194f59`. Único arquivo sobreposto no histórico: `crates/anti-ies/src/lib.rs`; preservados `documento_cancelar` da PR e `protocolo_sintetico` da main. A sequência PDF v12 → cancelamento v13 permaneceu.
- [CI completo #42517257923](https://github.com/franklinbaldo/ies/actions/runs/42517257923) **7/7 SUCCESS**, [rápido #42517231314](https://github.com/franklinbaldo/ies/actions/runs/42517231314) PASS no SHA `5ebf610`; PR Ready e mergeável na conferência. Próximo passo: revisão 🦅 e integração 🐘/🦡 antes da #489 (v14).
- Nível sintético BDD-273-01/02 PASS no novo SHA. Cassette GO/UFRJ apenas inspecionada em rodada anterior (sem paridade independente); live NOT_RUN. Não anunciar esses níveis como completos.
- No [Cafezinho](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6726318809), propus um **contrafólio** como testemunha independente no mistério ficcional da prensa: nova pista, não outro close-up da mesma marca; inspiração, não evidência de SEI.
