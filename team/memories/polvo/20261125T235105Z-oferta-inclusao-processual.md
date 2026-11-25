---
type: AgentMemory
title: "Retorno IES: oferta de inclusão processual em bloco"
description: "PR #881 DRAFT; pré-requisito funcional para BDD-381-05, sem POST novo."
agent: polvo
timestamp: "2026-11-25T23:51:05Z"
tags: [blocos, cli, cassette-first, anti-ies]
---

# Retomar

- Main de partida `97894c5d`; [PR #881](https://github.com/franklinbaldo/ies/pull/881) HEAD `d00dfebd`. Prévia de NUP ausente exige oferta inequívoca da barra raiz. Idempotência de NUP presente e bloqueio de confirmação ausente preservados.
- Card PEN1 `bloco-interno-processos-pen1--69795bb274d6.md` e oráculo Todos lidos. Hipótese sintética: oferta na unidade aberta. Acrescentados E2E negativo de CLI/SQLite, testes do leitor e asserção no replay. Rust/CI/E2E/replay original/prek NOT_RUN nesta rodada; live NOT_RUN. Nenhum POST novo ou SEI externo.
- [Coordenação #381](https://github.com/franklinbaldo/ies/issues/381#issuecomment-7178115673). Próximo: validar a forma real da oferta; lupa tipada, frmVisualizar e aceite causal 302 + releitura independente. Não disputar #876 (ciclo de vida), #872 (criação) ou #851 (editor).
