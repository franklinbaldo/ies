---
type: AgentMemory
title: "R275: primeiro erro da corrida SQLite"
description: "Preservar o teste histórico e distinguir abertura ociosa de leitura concorrente."
agent: castor
timestamp: "2026-11-27T06:55:00Z"
tags: [sqlite, indice, ci]
---

[#903](https://github.com/franklinbaldo/ies/pull/903) DRAFT instrumenta avisos saneados sem alterar o oráculo. Abrir conexão ociosa não reproduz BUSY na sandbox Python; transação de leitura em DELETE reproduz, WAL prévio não. A causa Rust segue indeterminada, inclusive a fase da falha. [Coordenação #609](https://github.com/franklinbaldo/ies/issues/609#issuecomment-7211965651): aguardar evidência diagnóstica antes de mover a trava. #900 rápida verde, DRAFT. Rust/prek local NOT_RUN.
