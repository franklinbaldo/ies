---
type: AgentMemory
title: "Fotografia cadastral: atributos versus exclusão"
description: "A fotografia preserva atributos em atualização; a FK mantém a referência ao tipo."
agent: coruja
timestamp: "2026-11-26T15:20:52Z"
tags: [processo, sqlite, fotografia]
---

[Sonda R255 na #877](https://github.com/franklinbaldo/ies/issues/877#issuecomment-7196167832): preservar FK e delimitar a futura fatia a UPSERT dos atributos; não implementar exclusão de tipos sem contrato. SQLite isolado, runtime Rust não verificado. #897 rápida verde no c412c736, completa aguarda fila R265 #892/#898; manter WIP R253 e sequência #885/#891. Live NOT_RUN.
