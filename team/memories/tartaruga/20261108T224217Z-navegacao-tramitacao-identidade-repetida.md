---
type: AgentMemory
title: "Identidade repetida na navegação de tramitação sem PDF"
description: "PR #515: RED comprovou árvore HTTP 200 indevida; validador compartilhado agora recusa duplicação com 400, preserva seleção vazia única e CI final 7/7."
agent: tartaruga
timestamp: "2026-11-08T22:42:17Z"
tags: [anti-ies, documento, navegacao, sqlite, bdd, cassette]
---

# Para retomar

Fato: [PR #515](https://github.com/franklinbaldo/ies/pull/515) corrige `procedimento_visualizar` e `procedimento_trabalhar` na navegação de tramitação **sem PDF**, onde dois `id_documento` eram ignorados e a árvore real do SQLite aparecia com 200. [RED oficial](https://github.com/franklinbaldo/ies/actions/runs/42531059294) evidenciou 1 teste falhando/4 passando; [CI final](https://github.com/franklinbaldo/ies/actions/runs/42531269385) fechou 7/7 no head `e55d8e0`. Teste técnico HTTP/SQLite, não novo E2E integral da CLI. O `id_documento=` vazio único continua válido.

Fato: [Actions #42531050048](https://github.com/franklinbaldo/ies/actions/runs/42531050048) conferiu integridade de duas cassettes UnB, 503 interações. **Não houve confronto semântico independente, REA nem live**. Conferir eventual avanço da main e encaminhar a PR para revisão/integração 🦅/🐘/🦡; deixar a issue #60 aberta. A conversa no [Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388) sobre testemunhas independentes é analogia útil, não evidência do SEI.
