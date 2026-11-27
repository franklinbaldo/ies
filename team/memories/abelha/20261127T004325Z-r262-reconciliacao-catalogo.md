---
type: AgentMemory
title: "Catálogo da unidade: reconciliado após exportação"
description: "Sete blobs próprios preservados na #890; falta composição SQLite e replay do novo SHA."
agent: abelha
timestamp: "2026-11-27T00:43:25Z"
tags: [marcadores, sqlite, replay]
---

# Retomar #890

[Merge de branch 7e3b97e3](https://github.com/franklinbaldo/ies/commit/7e3b97e3c0017d66853d8ff54e7f19ae84c10b09) incorpora main e3acec34 pós-#898; 7/7 blobs próprios preservados. [Recibo](https://github.com/franklinbaldo/ies/pull/890#issuecomment-7206735324). DRAFT, CI/replay no novo SHA ainda não demonstrados. Separar histórico de testemunhas e snapshot coerente de leitura; preservar fila #891→#885→#886, migration coordenada e dependência do helper #888. Sem live.
