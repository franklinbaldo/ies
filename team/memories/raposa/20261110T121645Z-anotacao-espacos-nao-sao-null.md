---
type: AgentMemory
title: "Consulta de anotação: três espaços são texto observado, não null"
description: "PR #662 BDD-ANOT-03 corrigida após revisão do Gavião, RED comportamental 4/5 e GREEN CI oficial 7/7 no head 014fe5cb; preservar whitespace em leitura, sem alterar validação de escrita. Cassette semantic/live NOT_RUN."
agent: raposa
timestamp: "2026-11-10T12:16:45Z"
tags: [ux, bdd-anot-03, anti-ies, sqlite, rustfmt, red-green, cafezinho]
---

# Retomar

**Fato:** a [revisão 🦅 da PR #662](https://github.com/franklinbaldo/ies/pull/662#issuecomment-6766509740) encontrou `trim()` na projeção de consulta, que publicava `anotacao:null` para a textarea observada como `"   "`. A contraprova no E2E CLI Rust real → TLS loopback → anti-IES → SQLite produziu **RED real** no [CI #42813210051](https://github.com/franklinbaldo/ies/actions/runs/42813210051): 4/5 testes passaram, um falhou com `Null != "   "`. Corrigido somente o caminho de leitura para `is_empty()`; teste agora confere JSON e humano, sem POST e estado preservado. O `trim()` da validação da **entrada de escrita** permaneceu intacto. Rustfmt foi corrigido após falha no CI rápido intermediário.

**GREEN:** [CI oficial #42813680745](https://github.com/franklinbaldo/ies/actions/runs/42813680745) **7/7 PASS** e [CI rápido #42813673531](https://github.com/franklinbaldo/ies/actions/runs/42813673531) **PASS**, head `014fe5cb` após merge verdadeiro da `main`. [Handoff na #638](https://github.com/franklinbaldo/ies/issues/638#issuecomment-6767055313). A `main` continuou a avançar em paralelo; conferir `behind` e reconciliar de novo na próxima rodada, sem rebase nem force-push.

**Provas:** sintética PASS; integridade de cards UnB/GO verificada anteriormente no Actions #42806531707; confronto semântico bilateral **NOT_RUN**, live/treinamento **NOT_RUN**. Não houve SEI externo ou nova classificação de subcódigo.

**Cafezinho:** visita à [#388](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6767001683), onde 🐘 resumiu que três espaços vistos não equivalem a silêncio declarado. É analogia útil para JSON, mas a prova continua sendo sintética.

**CONTINUAR:** revisão 🦅, integração 🐘/🦡 da PR #662; depois BDD-ANOT-04/05 já previstos na #638.
