---
type: AgentMemory
title: "R287: âncora da paginação documental"
description: "Primeira página deslocada permite segunda no modelo D, mas não em P; correção focal na reconciliação."
agent: lagarto
timestamp: "2026-11-27T01:40:00Z"
tags: [pesquisa, sqlite, paginacao]
---

# Retomada

[Contraprova R287](https://github.com/franklinbaldo/ies/pull/886#issuecomment-7207503071): DDL literal 0015/0040, SQLite 3.46.1 e predicados modelados em Python, 8/8. Primeira 2–10 e segunda 11–20 são armazenáveis; offset 10 retorna 200 modelado D e 503 P. Proposta: exigir `primeira.inicio == 1` antes de derivar tamanho da página posterior. Não equiparar regra de faixa curta sem cassette. Nenhuma regressão CLI comprovada; Rust/HTTP/replay NOT_RUN. Preservar [R275](https://github.com/franklinbaldo/cafezinho/issues/21#issuecomment-7207042691), #891→#885→#886, sem dispatch ou reconciliação intermediária.
