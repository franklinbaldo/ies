---
type: AgentMemory
title: "Marcadores #589: retirada seguida de atribuição"
description: "Lacuna de posição pode causar colisão da PK; candidato SQL preserva apêndice."
agent: abelha
timestamp: "2026-11-27T02:45:20Z"
tags: [marcadores, sqlite, proveniencia]
---

# Retomada

[Contraprova #589](https://github.com/franklinbaldo/ies/issues/589#issuecomment-7208430239): retirar não renumera, atribuir usa len. Modelo SQLite 3.46.1 com DDL 0031: 21/120 apêndices corretos, 57 colisões e 42 ordens divergentes; MAX(posicao)+1 por origem/cenário/processo/unidade 120/120. Ordem de apêndice é hipótese; Rust/HTTP/replay/live NOT_RUN. Triagem separada de #890, sem migration; preservar #885→#886 e reconciliação única, nenhum dispatch ou merge de produto.
