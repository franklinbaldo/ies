---
type: AgentMemory
title: "BDD-60-05: cancelamento não fornece conteúdo legível"
description: "RED real da CLI encontrou conteúdo servido após cancelamento; PR #529 corrige src nas duas árvores do anti-IES e tem CI completo 7/7; confronto semântico de cassettes permanece pendente."
agent: tartaruga
timestamp: "2026-11-09T02:49:36Z"
tags: [anti-ies, documento, cancelamento, bdd, cli, sqlite, cassette]
---

# Para retomar

[Fato, PR #529](https://github.com/franklinbaldo/ies/pull/529): o [RED #42552140652](https://github.com/franklinbaldo/ies/actions/runs/42552140652) mostrou a CLI real retornando 0 e pedindo documento_visualizar mesmo depois do cancelamento no SQLite. A projeção individual e mista agora não publica src em nó cancelado. A CLI já tinha recusa tipada, inclusive com --ocultar-inacessiveis. [CI final #42552655235](https://github.com/franklinbaldo/ies/actions/runs/42552655235): 7/7 verde no SHA ba688c5; PR Ready ainda aberta para Gavião/Elefante/Texugo.

[Cassettes #42552133055](https://github.com/franklinbaldo/ies/actions/runs/42552133055): GO documento-mover-cancelar (80 HTTP) e UnB documento-numero-e-busca (15 HTTP) verificadas por hashes/estrutura. **CLI↔anti-IES PASS sintético**, comparação semântica independente **NOT_RUN**, treinamento **NOT_RUN**, REA **NOT_RUN**.

O [Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388) continua discutindo como distinguir conclusões causais pela observação independente: analogia útil para não chamar um ícone de cancelamento de prova de indisponibilidade do corpo, mas não evidência do SEI. Retomar só lacuna real após a integração; não repetir BDD-60-01/02 já integrado.
