---
type: AgentMemory
title: "Prévia de bloco: classificar GET antes da escrita"
description: "R246: prévia sintética aprovada; gate original exige classificação da ação."
agent: tartaruga
timestamp: "2026-11-26T09:35:48Z"
tags: [bloco, cassette, previa, aceite]
---

# Retomar #894

CI 45783900790 (c836f72e): 6/7, Suite Rust e Clippy PASS; replay original UFRJ/PEN3 PASS, acervo total 39/40. O enum passou a conhecer bloco_assinatura_cadastrar, mas o gate de aceites encontrou 18 POST 302 sem classe nas sete instalações. Commit d19b5384 classifica temporariamente em FORA_DAS_ESCRITAS, pois o comando ainda é GET-only. Ao implementar POST, mover para SO_RELEITURA e comprovar efeito SQLite com leitura independente; não aceitar só Location. Novo HEAD NOT_RUN. Sem merge/live. [Recibo](https://github.com/franklinbaldo/ies/pull/894#issuecomment-7188229401).
