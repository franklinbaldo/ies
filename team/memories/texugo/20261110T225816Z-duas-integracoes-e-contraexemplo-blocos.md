---
type: AgentMemory
title: "Duas integrações verdes; hipótese multibloco exige contraprova"
description: "Integrei #703 (BDD-565-01 sintético) e #710 (regressão HTTP/SQLite), ambas 7/7; retenho #708 por possível exclusividade global indevida entre blocos."
agent: texugo
timestamp: "2026-11-10T22:58:16Z"
tags: [antichurn, integracao, ci, cassette-first, sqlite, bloco]
---

# Para retomar

**Fatos:** li `AGENTS.md`, ficha, guia cassette-first e cinco memórias recentes; reli o [Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388). Integrei [#703](https://github.com/franklinbaldo/ies/pull/703) no merge [`7b0ff04`](https://github.com/franklinbaldo/ies/commit/7b0ff04394e08732a24cfc8cceae93f7eadcdfb2): head `bd4ee5a`, [CI oficial 7/7](https://github.com/franklinbaldo/ies/actions/runs/42917844365), rápido PASS e 🦅 LGTM técnico. O BDD-565-01 tem PASS sintético CLI real ↔ anti-IES ↔ mesmo SQLite v38; [#565 atualizada](https://github.com/franklinbaldo/ies/issues/565#issuecomment-6781579789), BDDs 02–04 pendentes. A main só havia avançado em quatro memórias disjuntas.

Integrei [#710](https://github.com/franklinbaldo/ies/pull/710) no merge [`67d4c08`](https://github.com/franklinbaldo/ies/commit/67d4c081ceada84fa19e4f6c8f37cef06dff2332): head `e3c3626`, [CI oficial 7/7](https://github.com/franklinbaldo/ies/actions/runs/42918622007), rápido PASS e 🦅 LGTM técnico parcial sobre os mesmos quatro arquivos. Nova regressão é **HTTP direto ↔ anti-IES ↔ SQLite**, não E2E binário específico; [#496 atualizada](https://github.com/franklinbaldo/ies/issues/496#issuecomment-6781605225). Hashes de 75 HTTP UnB PASS, bilateral semântico e live NOT_RUN.

**Risco de código, ainda não contraexemplo executado:** em [#708](https://github.com/franklinbaldo/ies/pull/708), `documento_ja_observado_em_bloco` consulta sem número do bloco e `documento_arvore` suprime globalmente a oferta. Se DOC está em A, a inclusão legítima em B pode falhar; o manual UnB histórico admite multibloco. Registrei [veto motivado](https://github.com/franklinbaldo/ies/pull/708#issuecomment-6781570437): manter idempotência por bloco e testar A/B no mesmo SQLite, sem criar BDD, gate ou espera live. A [#526](https://github.com/franklinbaldo/ies/pull/526) permanece Draft/conflitante e guarda E2E que deve ser preservado.

Nenhum acesso ao SEI, credencial, subcódigo novo ou CI pós-merge. No Cafezinho, a distinção entre ação oferecida e estado comprovado segue útil: documento presente num bloco não prova exclusividade em outros.
