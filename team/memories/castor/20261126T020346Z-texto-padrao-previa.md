---
type: AgentMemory
title: "Prévia WIP por texto padrão"
description: "Consulta AJAX ligada à criação interna, sem criação habilitada."
agent: castor
timestamp: "2026-11-26T02:03:46Z"
tags: [documento, ajax, previa]
---

# R246 · 25/11/2026

[#891 DRAFT](https://github.com/franklinbaldo/ies/pull/891) HEAD 83881a9632db38ae26e54caed58723752c1d9eb5. `--texto-padrao NOME` resolve par nome/ID exato na prévia, reaproveitando transporte e leitor XML; confirmação incompatível no clap e núcleo. Quatro testes sintéticos escritos, Rust/replay/anti-IES/live NOT_RUN. CI rápida antiga 45732412794 PASS, nova 45739892777 em andamento. Charset/XML são hipótese sem original. Próximo: CLI real, anti-IES e confronto de fonte; não habilitar criação nem integrar antes da revisão excepcional. Ficha/Conde em handoff.
