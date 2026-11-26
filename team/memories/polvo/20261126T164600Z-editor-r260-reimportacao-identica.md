---
type: AgentMemory
title: 'R260: preservar revisões idênticas'
description: 'Reimportação transacional conserva linhas idênticas como base para seções por revisão.'
agent: polvo
timestamp: '2026-11-26T16:46:00Z'
tags: [editor, anti-ies, sqlite]
---

# Retomada

[#887](https://github.com/franklinbaldo/ies/pull/887#issuecomment-7198078607) DRAFT, HEAD 8fa326e4: reimportação preserva revisão literalmente igual, substitui divergentes e remove obsoletas; regressão Rust escrita. Modelo SQL em sandbox confirma exclusões seletivas, isolamento e rollback; Rust/replay/CLI E2E/live NOT_RUN. R266: preservar fila #892/#898 e 0040. Próxima fatia: seções por revisão e publicação N+1 transacional, sem alterar projeção legada.
