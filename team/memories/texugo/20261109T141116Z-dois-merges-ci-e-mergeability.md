---
type: AgentMemory
title: "Dois merges: diagnóstico da prévia e identidade UnB"
description: "Integrei #573 e #572 com CI 7/7 no head; #526 segue pendente em SQLite v22; recálculo do mergeable evitou falso bloqueio."
agent: texugo
timestamp: "2026-11-09T14:11:16Z"
tags: [antichurn, integracao, cassettes, cli, sqlite]
---

# Retomar

As PRs [#573](https://github.com/franklinbaldo/ies/pull/573) e [#572](https://github.com/franklinbaldo/ies/pull/572) foram integradas em sequência, merges `74540ad` e `22fd1cf`. Cada head exato teve CI oficial **7/7** e rápido verdes antes da integração; **não** houve CI pós-merge. A #573 cruza o `id_documento` do histórico com o GET original de conteúdo UnB, derivando número/tipo observados para replay HTTP/SQLite; **cassette PASS parcial**, sem novo E2E do binário nem live. A #572 preserva diagnóstico tipado saneado de prévia em script de ensaio; na UnB, houve login com seis requisições e **prévia FAIL**, sem POST de atribuição. O patch não resolve a falha nem demonstra novo PASS live.

Uma consulta inicial retornou `mergeable=false` para #572, mas o GET GitHub mostrou `mergeable=null / unknown`; nova leitura resolveu para `true / clean`, permitindo merge sem intervenção na branch alheia. Antes de tratar `false` de metadados como conflito concreto, confirmar o estado atualizado.

Única PR restante: [#526](https://github.com/franklinbaldo/ies/pull/526), Draft/conflitante, da 🦉. A criação causal deve reconciliar a `main` e a migração SQLite **v22** (v21 já ocupada), preservando E2E e obtendo gates no novo head. Já orientada na [PR](https://github.com/franklinbaldo/ies/pull/526#issuecomment-6740676553), não repetir comentário sem mudança.

Li o [Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6741636030). A identidade entre testemunhas históricas precisa de ligação observada; essa analogia ajuda a evitar extrapolar um replay parcial para uma causalidade de escrita que não foi gravada.
