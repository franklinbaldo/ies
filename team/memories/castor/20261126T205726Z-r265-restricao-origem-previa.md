---
type: AgentMemory
title: "R265: restrição de origem da prévia T"
description: "Reparo R270 e validações pendentes no novo candidato."
agent: castor
timestamp: "2026-11-26T20:57:26Z"
tags: [documento, previa, ajax, ci]
---

# Retomada

[R270](https://github.com/franklinbaldo/ies/pull/891#issuecomment-7202359183) aprovou hipótese operacional exclusivamente treinamento/bancada. [#891](https://github.com/franklinbaldo/ies/pull/891#issuecomment-7203291177) DRAFT, HEAD 1fd73d3a76454ecf5a4f3e19ee643bf9ce744b3c, commit 1/0 sobre 2bf1b91d, sete arquivos +94/-6, readback 7/7 e ordem estática conferida. Reuso de exigir_origem, CLI antes de ler sessão, núcleo antes de rede; N, recusa T+confirmar e D indisponível preservados no diff. Const de efeito cita hipótese, sem alterar derivador. Testes novos: origem produção/imitadora na CLI antes de arquivos de sessão (ausentes) e núcleo zero pedidos; positivo roteirizado passa a usar sei.test, CLI anti-IES anterior mantido. Contrato 19.805 bytes. [Rápida 45894971164](https://github.com/franklinbaldo/ies/actions/runs/45894971164) em execução na consulta; Rust/Clippy/replay locais NOT_RUN (toolchain ausente), oficial após #898, FINAL próprio solicitado. [Cafezinho](https://github.com/franklinbaldo/cafezinho/issues/21#issuecomment-7203291931); [Gavião](https://github.com/franklinbaldo/gaviao/issues/182#issuecomment-7203292685). Sem merge, dispatch próprio, SEI live ou mudança DDL. Prek/OKF pessoal NOT_RUN, sem checkout sincronizado. Ficha/Conde handoff.
