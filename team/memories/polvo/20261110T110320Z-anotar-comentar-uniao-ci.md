---
type: AgentMemory
title: "Anotar e Comentar coexistem após reconciliação de ramos"
description: "PR #654 no head 3f10794d, CI 7/7; três merges verdadeiros preservaram a anotação e o novo subcomando Comentar, com provas sintética, cassette e live distintas."
agent: polvo
timestamp: "2026-11-10T11:03:20Z"
tags: [bdd638, anotacao, comentarios, anti-ies, merge, cli, ci, cassette-first]
---

# Lembrete para a próxima rodada

- **Fato:** [PR #654](https://github.com/franklinbaldo/ies/pull/654) foi reconciliada com a main por merges verdadeiros [`a4c494b`](https://github.com/franklinbaldo/ies/commit/a4c494b55ff868c7abbff702a63e8b7289ecb51c), [`cc583cd`](https://github.com/franklinbaldo/ies/commit/cc583cd0e35e9c66f4157ddae8aa93ef608a1b43) e [`3f10794`](https://github.com/franklinbaldo/ies/commit/3f10794d98cbdff943fc997cf1db95e2b031f800). Em `src/cli_processo.rs`, preservar **ambos** os subcomandos `Comentar` da main e `Anotar --confirmar` da fatia; a árvore final divergia da main só nos dez arquivos originais da PR.
- **Fato:** [CI oficial #42803659062](https://github.com/franklinbaldo/ies/actions/runs/42803659062) **7/7 PASS** e [rápido #42803618301](https://github.com/franklinbaldo/ies/actions/runs/42803618301) PASS no head `3f10794d`. PR Ready/mergeável e `behind=0` após o CI, antes desta memória. O registro direto na main pode criar uma nova defasagem documental; verificar SHAs sem invalidar a evidência antiga.
- **Limites:** CLI real ↔ anti-IES ↔ SQLite PASS sintético; integridade das 77 interações UnB/GO previamente verificada, mas POST/releitura semânticos bilateral **NOT_RUN**; treinamento/live **NOT_RUN**, sem escrita externa. [Handoff #638](https://github.com/franklinbaldo/ies/issues/638#issuecomment-6765656665).
- **Cafezinho:** [comentário](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6765591807) sobre não apagar uma capacidade legítima ao resolver um conflito de enum. É uma lição de reconciliação do código, não evidência nova de cassette.
- **Retomar:** se #654 já integrada, iniciar BDD-ANOT-03 da #638; se aberta, consultar revisão, mergeability e CI antes de editar. Não criar novos BDDs.
