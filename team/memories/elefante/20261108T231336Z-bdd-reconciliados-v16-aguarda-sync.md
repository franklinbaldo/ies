---
type: AgentMemory
title: "Checklists BDD reconciliados; v16 pronta, atrás da memória"
description: "Os checklists #61, #333 e #246 estavam desatualizados apesar das integrações; #491 tem CI 7/7, mas uma memória OKF nova reabriu behind=1."
agent: elefante
timestamp: "2026-11-08T23:13:36Z"
tags: [integracao, bdd, antichurn, sqlite, memoria]
---

# Para a próxima rodada

**Fatos:** atualizei os BDDs já integrados nas issues [#61](https://github.com/franklinbaldo/ies/issues/61) (01–03), [#333](https://github.com/franklinbaldo/ies/issues/333) (01–05), [#246](https://github.com/franklinbaldo/ies/issues/246) (01–04) e #60 (04). A fila está no [roadmap #56](https://github.com/franklinbaldo/ies/issues/56#issuecomment-6728503655). Evitar refazer BDDs porque o checkbox histórico ficou pendente; fonte de verdade é PR integrada com CLI testada.

**Ponto de retomada:** [#491](https://github.com/franklinbaldo/ies/pull/491#issuecomment-6728488579) está Ready e teve [7/7 oficiais](https://github.com/franklinbaldo/ies/actions/runs/42533498988) no SHA `9dedb29e`, com SQLite v16 correto; uma memória 🦫 na `main` criou `behind=1`. Aguardar merge verdadeiro da `main` na branch pelo dono e novo CI do SHA final; então integrar. Depois, BDD-258-02/03/04. #60-01/02/05 ainda precisam conferência por contratos/testes, sem bloqueio por cassette/REA/live.

**Cafezinho:** [conversa sobre acorde final e pistas isoladas](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6728506457) lembra, como analogia artística e não evidência do SEI, que o mesmo estado final pode esconder ordens causais distintas.
