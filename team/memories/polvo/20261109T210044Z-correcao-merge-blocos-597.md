---
type: AgentMemory
title: "Correção aditiva: blocos PR #597 integrados, próximo BDD-256-02"
description: "A PR #597 foi mesclada à main após o CI 7/7; a memória anterior refletia um head ainda aberto. A issue #256 segue para paginação e detalhe."
agent: polvo
timestamp: "2026-11-09T21:00:44Z"
tags: [blocos, bdd256, merge, anti-ies, correcao]
---

# Estado corrigido após a corrida de integração

- **Fato posterior:** a [PR #597](https://github.com/franklinbaldo/ies/pull/597) foi mesclada em 09/11/2026 às 20:53:18 UTC, [merge `7ef044bb`](https://github.com/franklinbaldo/ies/commit/7ef044bbcc1a17dff57ed23ffe95fb6a12cc4a2e), com [CI 7/7 #42696723917](https://github.com/franklinbaldo/ies/actions/runs/42696723917) no head `3fab8734`. Isso corrige o estado de integração descrito na [memória anterior](20261109T205454Z-ci-blocos-corrida-memorias.md), sem reescrevê-la.
- **Provas:** BDD-256-01 sintético PASS; confronto independente dos HTMLs originais com IES e anti-IES NOT_RUN (só integridade dos 324 registros UnB/GO verificada); treinamento NOT_RUN, sem requisições externas. As provas não se substituem.
- **Retomar:** [issue #256](https://github.com/franklinbaldo/ies/issues/256#issuecomment-6751341398) aberta para BDD-256-02 e BDD-256-03. Antes de implementar a página 2 dos blocos internos, ler o membro original da cassette no GitHub Actions; o card GO atesta 29 POST mas não qual corpo corresponde à paginação. Evitar nova PR baseada em migração v27 até reconciliar o trabalho concorrente da #602. A visita ao [Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388) ocorreu nesta rodada.
