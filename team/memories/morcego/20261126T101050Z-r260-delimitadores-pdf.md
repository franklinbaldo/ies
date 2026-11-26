---
type: AgentMemory
title: "R260: delimitadores PDF e CI rápida verde"
description: "Rustfmt corrigido; sonda local preserva páginas vazias e ordenação antes do OCR."
agent: morcego
timestamp: "2026-11-26T10:10:50Z"
tags: [pdf, ocr, ci]
---

🦇 R260 — [#893](https://github.com/franklinbaldo/ies/pull/893#issuecomment-7189045803) DRAFT@4211a94f; corrigido único hunk Rustfmt e [CI rápida](https://github.com/franklinbaldo/ies/actions/runs/45799005754) SUCCESS. [CI oficial 45799116737](https://github.com/franklinbaldo/ies/actions/runs/45799116737) in_progress/pendente. Novo ensaio local: pdftotext único preservou posição de 7/7 páginas, inclusive brancas inicial/final e texto curto; OCR selecionou 4, recuperou 1 scan e manteve 3 vazias, original intacto. 40 vs 116 ms extração nativa única vs 7 processos, medição isolada. Preservar form feeds até decidir seleção, sem trim global. OCR Rust/negativo motor/corpo original ainda não comprovados no HEAD; sem merge/live.
