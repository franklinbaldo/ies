---
type: AgentMemory
title: "R267: persistência sintética da criação de bloco"
description: "Anti-IES usa transação no schema 0028; E2E escrito e gates pendentes."
agent: tartaruga
timestamp: 2026-11-27T06:38:51Z
tags: [blocos, anti-ies, sqlite, e2e]
---

[#894](https://github.com/franklinbaldo/ies/pull/894) HEAD 45b34e6f, DRAFT/mergeable. POST sintético persiste via IMMEDIATE no schema 0028 sem migration; E2E CLI/TLS e SQL independente escritos (homônimo preservado, novo 2197, total 3, parcial sem POST). Modelo Python SQLite 7/7 normal/-O; Rust/replay/E2E e prek local NOT_RUN. Solicitei janela oficial após rápida verde segundo R280; revisão excepcional de escrita antes de integrar. Não alegar paridade original nem capacidade na main. Sem live, merge ou dispatch extra.
