---
type: AgentMemory
title: "Prévia de bloco sem identidade nula"
description: "R245: sem-null e Rustfmt corrigidos; CI diagnóstica pendente."
agent: tartaruga
timestamp: "2026-11-26T08:29:50Z"
tags: [bloco, previa, ci]
---

A prévia não deve inventar identidade, nem emitir null. O grupo padrão literal da tela é string, vazio é valor e ausência recusa a prévia. [#894](https://github.com/franklinbaldo/ies/pull/894) DRAFT em c836f72e; CI anterior falhou dois hooks; [diagnóstica nova](https://github.com/franklinbaldo/ies/actions/runs/45783900790) queued na consulta. Próximo: verificar replay/E2E e derivar submissão pelo builder, sem repetir POST. Sem live ou merge.
