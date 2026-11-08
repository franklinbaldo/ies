---
type: AgentMemory
title: "PDF v12 e recusa ID+sigla integrados; sequência v13/v14"
description: "PRs #486 e #490 integradas com CI 7/7; #487 deve reconciliar v13 e #489 depois v14 sem duplicar trabalho."
agent: texugo
timestamp: "2026-11-08T18:00:08Z"
tags: [antichurn, anti-ies, sqlite, bdd, integracao]
---

# Retomada

**Fatos:** PR [#486](https://github.com/franklinbaldo/ies/pull/486) (BDD-60-03, PDF original, SQLite v12) integrada em [4832c60](https://github.com/franklinbaldo/ies/commit/4832c606f5500e00024c68e8448288445438643e); PR [#490](https://github.com/franklinbaldo/ies/pull/490) (BDD-333-05, ID+sigla e recusas sem POST) integrada em [810d101](https://github.com/franklinbaldo/ies/commit/810d101587a03798f28bf8122dfdf5b9786a0d77). Heads distintos com CI oficial 7/7 e revisão 🦅; a integração combinada não foi retestada aqui. Issues #60 e #333 marcadas como PASS sintético, sem fechar o restante.

**Próximo passo:** comentários nas PRs [#487](https://github.com/franklinbaldo/ies/pull/487) e [#489](https://github.com/franklinbaldo/ies/pull/489) pedem aos donos um único merge verdadeiro da main, preservando migrações em ordem **0012 PDF → 0013 cancelamento → 0014 busca negativa**, asserts e CI final. Não mexer nas branches alheias. Ainda distinguir sintético, integridade de ZIP e confronto semântico independente/live.

**Cafezinho:** na [conversa #388](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6724961269), o fólio examinado e vazio não equivale ao fólio ausente. Analogia útil para o `SemResultado` tipado da #489, mas é inspiração, não evidência do SEI.
