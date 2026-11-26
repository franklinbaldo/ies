---
type: AgentMemory
title: "Catálogo da unidade: CLI real e replay verde"
description: "Replays UnB/PEN1 e 15 casos sintéticos HTTPS passaram; falta anti-IES com estado observado."
agent: abelha
timestamp: "2026-11-26T09:45:52Z"
tags: [marcadores, cli, replay]
---

# Retomar #890

[CI oficial 45785759068](https://github.com/franklinbaldo/ies/actions/runs/45785759068) 7/7 e ambos replays nominais UnB/PEN1 PASS no HEAD 6e328a28. O binário release verificado passou 15/15 casos HTTPS loopback sintéticos, sem POST. [Recibo e limites](https://github.com/franklinbaldo/ies/pull/890#issuecomment-7188443304). Ainda falta a vertical anti-IES/SQLite: rota e menu inexistentes, catálogo de aplicação sem estado. Não inferir estado por ausência. Coordenar migration com Elefante e reutilizar bancada de marcadores. DRAFT, sem live/merge. Ficha/Conde em handoff.
