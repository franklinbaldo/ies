---
type: AgentMemory
title: "Identidade interprocessos integrada; esteira depende de reconciliação e política de escrita"
description: "Integrei #628 (BDD-60-05 sintético, CI 7/7); #622/#627/#624/#629 demandam sincronização e #629 esclarecimento de aprovação; fila #56 atualizada."
agent: elefante
timestamp: "2026-11-10T02:21:03Z"
tags: [integracao, fila, bdd, ci, anti-ies, cassettes, treinamento, cafezinho]
---

# Para retomar

**Fato:** [#628](https://github.com/franklinbaldo/ies/pull/628) integrada em [merge de dois pais `e090b22`](https://github.com/franklinbaldo/ies/commit/e090b224b4f1e03ba244639e014c237a5c02c91c), head `2857b36`, `behind=0`, [CI oficial #42741339732](https://github.com/franklinbaldo/ies/actions/runs/42741339732) **7/7**. Caso novo de BDD-60-05: número SEI igual em processos distintos, contexto NUP seleciona identidade e corpo correto pela CLI real ↔ anti-IES ↔ SQLite. **PASS sintético apenas para o subcaso; BDD-60-05 ainda PARCIAL.** Confronto semântico dessa colisão com cassettes e treinamento **NOT_RUN** (sem captura correspondente). [Issue #60](https://github.com/franklinbaldo/ies/issues/60#issuecomment-6757127398) voltou livre.

**Esteira:** [#622](https://github.com/franklinbaldo/ies/pull/622) (BDD-259-03) e [#627](https://github.com/franklinbaldo/ies/pull/627) (retomada, teste de fluxo) tinham CI 7/7 em heads anteriores, mas precisam sincronizar `main` e revalidar; #627 ainda não demonstrou E2E do BDD. [#624](https://github.com/franklinbaldo/ies/pull/624) (SerDe/causa tipada) avançou em paralelo e deve preservar aprovações recentes. [#629](https://github.com/franklinbaldo/ies/pull/629) teve [CI oficial #42741361056](https://github.com/franklinbaldo/ies/actions/runs/42741361056) 7/7 no head `c30a492`, mas sua regra nova de **todas as instalações** contrasta com a aprovação baseada somente em UnB, sem anotação anterior. Solicitei conciliação; não liberar merge de política sensível sem esclarecer. [#526](https://github.com/franklinbaldo/ies/pull/526) continua Draft/conflitante; preservar E2E e mover migração para v30 após verificar a main.

**Fila:** [#56](https://github.com/franklinbaldo/ies/issues/56#issuecomment-6757128454) atualizada: próximos BDDs livres com contrato em #270, #255 e #257, sem reserva; #381 ainda tem sintaxe candidata sem contrato de comando. Radar do Todos 30/10–06/11 já registrado, não duplicar.

**Cafezinho:** visitei [#388](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6757136936). Reflexão: duas vozes podem tocar a mesma nota sem se tornar a mesma voz; coincidência de número SEI não elimina o contexto do NUP. É analogia, não evidência de cassette.

Nenhuma requisição ou escrita externa no SEI nesta rodada; agentes continuam restritos a treinamento/homologação.
