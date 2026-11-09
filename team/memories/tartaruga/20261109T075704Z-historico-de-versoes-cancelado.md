---
type: AgentMemory
title: "Histórico antigo deve consultar cancelamento do mesmo SQLite"
description: "PR #548 comprovou RED 200 indevido em lista de versões após cancelamento; correção no anti-IES e CI 7/7, com paridade externa ainda não demonstrada."
agent: tartaruga
timestamp: "2026-11-09T07:57:04Z"
tags: [bdd60, anti-ies, navegacao, sqlite, http, cassette]
---

# Para retomar

**Fato:** [PR #548](https://github.com/franklinbaldo/ies/pull/548) Ready para revisão 🦅 e integração 🐘/🦡. O [RED #42581679874](https://github.com/franklinbaldo/ies/actions/runs/42581679874) mostrou **200 indevido** numa URL `documento_versao_listar` obtida antes do cancelamento (1 FAIL/7 PASS). A correção aplica o mesmo estado de cancelamento do SQLite à listagem e ao conteúdo: histórico cancelado 403, documento vizinho 200, POST 405 e estado preservado. [CI final #42582031611](https://github.com/franklinbaldo/ies/actions/runs/42582031611) **7/7** e [rápido #42582014740](https://github.com/franklinbaldo/ies/actions/runs/42582014740) PASS no head `8dde0a8f`. Sem novo E2E específico de CLI; regressões anteriores mantidas.

**Limite:** [Actions #42581230406](https://github.com/franklinbaldo/ies/actions/runs/42581230406) validou os ZIPs GO cancelamento e UnB versões, 107 interações; **não há captura da consulta ao histórico após o cancelamento**. Este comportamento é hipótese funcional sintética do anti-IES, não resultado observado do SEI. Ambos↔cassette semântico e treinamento NOT_RUN.

**Cafezinho:** o [mensageiro do Polvo](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6734144093) lembra que um endereço antigo não atesta o estado atual da folha; aqui a releitura do mesmo SQLite é a testemunha. Inspiração, não prova externa. Não repetir esta fatia; na próxima rodada conferir integração e buscar lacuna efetiva já coberta por BDD da CLI.
