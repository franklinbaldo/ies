---
type: AgentMemory
title: "POST sintético sem Content-Length não alcança rota"
description: "PR #493 falhou em 405 esperado versus 400 real porque o helper omitiu Content-Length em POST vazio; orientar ajuste só no teste e preservar a correção funcional."
agent: texugo
timestamp: "2026-11-08T18:55:42Z"
tags: [antichurn, testes, http, ci, anti-ies, ownership]
---

# Retomar sem retrabalho

**Fato:** no [CI oficial #42512114185](https://github.com/franklinbaldo/ies/actions/runs/42512114185), PR #493, a regressão HTTP falhou ao esperar 405 no POST. O helper `pedir()` omite `Content-Length`; `ler_pedido` devolve 400 antes da rota. Orientei adicionar `Content-Length: 0` ao POST do teste, sem mexer na produção ([comentário](https://github.com/franklinbaldo/ies/issues/493#issuecomment-6725652692)). Os testes da CLI interprocessos passaram nesse run; ainda falta CI completo verde no head corrigido.

**Coordenação:** #492 tem aprovação técnica e GREEN 7/7 anterior, mas a main avançou inicialmente apenas com memórias. Pedi sincronização por merge verdadeiro e novo CI final ([comentário](https://github.com/franklinbaldo/ies/issues/492#issuecomment-6725627358)). Marquei #60 🐢, #258 🦫 e #273 🐙 porque já possuem PRs ativas, sem reservar trabalho futuro. Na sequência de SQLite, preservar #487 v13 → #489 v14 → #491 v15; não refazer as implementações.

**Cafezinho:** a [discussão #388](https://github.com/franklinbaldo/ies/issues/388) distingue folha examinada e vazia de folha ausente. É inspiração útil para a busca negativa explícita da #489, **não** prova sobre SEI. Continuam separados PASS sintético, inspeção estrutural de cassette e paridade externa/live.
