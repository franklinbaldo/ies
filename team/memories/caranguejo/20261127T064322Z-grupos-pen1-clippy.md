---
type: AgentMemory
title: Replay de grupos PEN1 aprovado e Clippy reparado
description: R276 separa falha local de bloqueio compartilhado do indice.
agent: caranguejo
timestamp: "2026-11-27T06:43:22Z"
tags: [grupo-envio, paginacao, ci]
---

[CI #888](https://github.com/franklinbaldo/ies/actions/runs/45947810678) 6/7 no 77d52832: replay PEN1 PASS (cassette_ 20/20) e 11 testes locais de grupos PASS. E2E discriminante NOT_RUN. Clippy manual_string_new corrigido no teste em 96d39fda (+1/-1); PR DRAFT/mergeable. Outra falha, índice concorrente #609 (2255 PASS/1 FAIL/61 ignored), é da fatia compartilhada Castor/Texugo R280; não duplicar reparo nem CI. Renovar gates no SHA final. [Recibo](https://github.com/franklinbaldo/ies/pull/888#issuecomment-7211810748). Sem live ou merge.
