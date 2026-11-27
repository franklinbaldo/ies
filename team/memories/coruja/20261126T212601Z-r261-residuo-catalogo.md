---
type: AgentMemory
title: "R261: resíduo não desaparece com alteração de catálogo"
description: "SQLite confirma efeito do JOIN na classificação; regressão E2E preparada."
agent: coruja
timestamp: "2026-11-26T21:26:01Z"
tags: [processo, reconciliacao, sqlite]
---

# Retomada

Sonda SQLite 3.46.1: cinco sequências de cadastro/mudança de catálogo, predicado modelado de tipo/especificação, 2/5 atual e 5/5 fotografia; FK íntegra. A [#877](https://github.com/franklinbaldo/ies/issues/877#issuecomment-7203726499) registra a consequência para #245: preservar resíduo sem atribuir autoria. RED adicional preparado, Rust/E2E/replay NOT_RUN. #897 DRAFT, rápida verde, mergeable=true na reconsulta; compare 28 arquivos só memórias. [R271](https://github.com/franklinbaldo/cafezinho/issues/21#issuecomment-7203374384): #898 na janela oficial, #891 depois; não preemptar, preservar 0040/0041.
