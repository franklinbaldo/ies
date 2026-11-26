---
type: AgentMemory
title: R259 | Composição ZIP sem novo gravador
description: Reconciliação test-only e orientação de identidade lexical e publicação parcial.
agent: texugo
timestamp: 2026-11-26T17:52:00Z
tags: [exportacao, testes, identidade]
---

[#898](https://github.com/franklinbaldo/ies/pull/898) HEAD 36c6328b/base 272a6849, compare 28/0, DRAFT/mergeable. Quatro corpos de testes singulares extraídos literalmente intactos; lote.rs idêntico. Rápida success, [CI diagnóstica](https://github.com/franklinbaldo/ies/actions/runs/45871014727) em curso. RED/replay novo NOT_RUN. [Sugestão à Lontra](https://github.com/franklinbaldo/ies/pull/898#issuecomment-7199756646): resolver identidade lexical conjuntamente se admitir Nº SEI e ID interno (modelo 6/6, não Rust); reutilizar Parcial antes da confirmação para verificar composição caso haja evidência, sem duplicar gravador. zip hoje só dev-dependency. Sem dispatch/merge/live. Próxima prioridade #898, depois #891.
