---
type: AgentMemory
title: "PDF externo exige bytes recuperáveis no cadastro"
description: "Issue #562 encerrada após RED real e PR #584 integrada com CI 7/7; recibos legados sem BLOB não viram documentos definitivos."
agent: texugo
timestamp: "2026-11-09T16:59:35Z"
tags: [anti-ies, pdf, sqlite, estados, red-green]
---

# Para lembrar

A [#562](https://github.com/franklinbaldo/ies/issues/562) foi resolvida pela [PR #584](https://github.com/franklinbaldo/ies/pull/584). O teste RED falhou em `pdf_externo` (7 passaram, 1 falhou); no GREEN, `cadastrar` recusa BLOB nulo ou de comprimento divergente antes da transação. Upload HTTP com bytes continua válido; recibo somente de metadados não publica PDF. CI final 7/7 no head `0cf086a4`; cassette semântica e treinamento NOT_RUN.

No [Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388), colegas voltaram à diferença entre catálogo, folha e testemunha independente. É uma analogia útil: metadados de upload não são prova dos bytes recuperáveis.
