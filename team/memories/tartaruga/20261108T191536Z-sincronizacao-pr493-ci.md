---
type: AgentMemory
title: "PR #493: sincronização por merge verdadeiro e CI no novo head"
description: "BDD-60-04/05 já corrigidos; incorporadas duas memórias da main na PR #493 via merge de dois pais, preservando código e reacionando gates no SHA novo."
agent: tartaruga
timestamp: "2026-11-08T19:15:36Z"
tags: [anti-ies, navegacao, bdd, integracao, ci]
---

# Para a próxima rodada

[PR #493](https://github.com/franklinbaldo/ies/pull/493) continha teste de CLI real para BDD-60-04 e recusa HTTP 400 sem fallback para identificador duplicado, com 7/7 verde no head anterior `32072f2`. A `main` avançou duas memórias OKF sem tocar em código; fiz merge verdadeiro de dois pais `8c9d603` preservando as duas adições e mantive as três mudanças funcionais da PR. Reativei o CI oficial no novo head; conferir seu resultado antes de integração, sem atribuir o verde anterior ao SHA novo.

Cassette UnB: apenas hashes e estrutura conferidos anteriormente; confronto semântico independente e treinamento permanecem não executados. No [Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388), a discussão sobre folha examinada/vazia versus fólio ausente lembra que ausência não é evidência negativa. Não é prova sobre SEI. Retomar a PR, não duplicar BDD já implementado.
