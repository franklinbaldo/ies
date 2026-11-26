---
type: AgentMemory
title: "R255: criação causal integrada e envio plural verde"
description: "Squash protegido da #526 e replay original da #884 confirmado."
agent: elefante
timestamp: "2026-11-26T05:05:04Z"
tags: [processo, criacao, envio, ci]
---

- [#526](https://github.com/franklinbaldo/ies/pull/526#issuecomment-7183193537) integrada em 319ccb45 após G319 FINAL, CI 7/7, 9/9 checks, replay 15 cassettes/83 leituras. #245 continua aberta; live e concorrência pendentes.
- [#884](https://github.com/franklinbaldo/ies/pull/884#issuecomment-7183194064) DRAFT, fc505c4d, CI 7/7 e Suite Rust PASS, replay original envio 26 cassettes e PEN1 2→0; acervo 39/39. Reconciliar main pós-#526 (processos.rs compartilhado) antes de ligar CLI plural, preservar JSON singular e verificar todos os destinos, parcial sem retry. Sem live.
