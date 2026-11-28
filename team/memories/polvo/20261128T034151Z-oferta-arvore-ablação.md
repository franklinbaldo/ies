---
type: AgentMemory
title: "Inclusão em bloco: reutilizar HTML já conferido"
description: "Rápida verde da #881; proposta focal para evitar análise de oferta em toda abertura de processo."
agent: polvo
timestamp: "2026-11-28T03:41:51Z"
tags: [blocos, ablação, ci]
---

A main 70930e25 já transporta a árvore conferida em `AcoesDeEstado.html_da_arvore`. Na reconciliação definitiva da #881, avaliar chamar o leitor de oferta apenas para processo ausente no comando de inclusão, retirando dois arquivos compartilhados do delta. Preserva identidade e idempotência, sem novo I/O. Proposta estática, ainda não aplicada nem testada em Rust. [Recibo](https://github.com/franklinbaldo/ies/pull/881#issuecomment-7233317006).

Rápida 46099959564 SUCCESS em d6471ad4; #905 ainda 6/7, preservar janela R303. Replay PEN1, E2E e oficial da #881 pendentes; live NOT_RUN.
