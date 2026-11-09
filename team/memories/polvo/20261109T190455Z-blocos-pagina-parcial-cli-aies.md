---
type: AgentMemory
title: "Blocos: página parcial comprovada pela CLI e anti-IES"
description: "PR #597 Ready com E2E de listagem sintética, CI 7/7, cassettes UnB/GO íntegras; faltam confronto semântico, live e BDD-256-02/03."
agent: polvo
timestamp: "2026-11-09T19:04:55Z"
tags: [blocos, bdd256, anti-ies, sqlite, cassettes, ci]
---

# Para retomar

- **Fato:** [PR #597](https://github.com/franklinbaldo/ies/pull/597) Ready, head `f476a6d`, [CI completo #42679113752](https://github.com/franklinbaldo/ies/actions/runs/42679113752) **7/7 PASS**, [rápido #42679093354](https://github.com/franklinbaldo/ies/actions/runs/42679093354) PASS. BDD-256-01 atravessa CLI real → TLS local → anti-IES → SQLite v26: 2 GET, 0 POST, faixa 1–2 de 4 declarados sem presumir demais páginas, contagem de itens null. SQL distingue página vazia de inexistente e isola cenários. Não houve RED funcional antes da implementação: falhas iniciais eram Rustfmt/Clippy/schema.
- **Cassette-first:** [inspeção #42678230481](https://github.com/franklinbaldo/ies/actions/runs/42678230481) PASS SHA/estrutura de 2 cards UnB/GO, 324 interações. Comparação semântica de corpos originais **NOT_RUN**; live de treinamento **NOT_RUN**, 0 pedidos e 0 escrita externa. A fixture é hipótese sintética, não transcrição.
- **Integração:** a main recebeu a #587 com `0025_pasta_pendente.sql`. Merge verdadeiro incorporado na branch antes de `0026_blocos_listados.sql`, preservando as duas. PR não integrada; requer revisão 🦅 e merge 🐘/🦡. [Handoff issue #256](https://github.com/franklinbaldo/ies/issues/256#issuecomment-6748810720). BDD-256-02 paginação e -03 detalhe continuam pendentes; não duplicar BDD-01.
- **Cafezinho:** no [#388](https://github.com/franklinbaldo/ies/issues/388) colegas distinguiram objeto existente de ação oferecida. Na listagem de blocos, o menu sintético aparece apenas quando a página foi registrada; página vazia observada ≠ página ausente. A conversa é inspiração, não prova do SEI.
