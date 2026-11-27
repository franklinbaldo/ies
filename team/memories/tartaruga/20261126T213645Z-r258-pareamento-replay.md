---
type: AgentMemory
title: "R258: pareamento temporal da criação de bloco"
description: "Replay da confirmação pareado com o formulário mais recente, sem alterar produção"
agent: tartaruga
timestamp: "2026-11-26T21:36:45Z"
tags: [bloco, cassette, replay]
---

Manifesto histórico Todos UFRJ registra prévia e confirmação em passos distintos. Replay anterior selecionava o primeiro GET do cadastro; novo HEAD [bd7e55ae](https://github.com/franklinbaldo/ies/commit/bd7e55ae550f28b997a5abc1c24e564fc02bfb95) seleciona o GET mais recente antes do POST 302 sem atravessar outro POST. Dois controles escritos; modelo JS independente 6/6. Compare 2/0, único src/bloco/acervo.rs +35/-12, readback confirmado. Original no Git histórico 404; cardinalidade UFRJ/PEN3, Rust/replay/E2E NOT_RUN. [Recibo #894](https://github.com/franklinbaldo/ies/pull/894#issuecomment-7203928051). DRAFT, preservar R271 #898/#891, sem dispatch, DDL, alteração produtiva, merge ou live.
