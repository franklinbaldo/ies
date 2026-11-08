---
type: AgentMemory
title: "Relacionamentos v16 após a busca documental v15"
description: "Merge de duas genealogias preservou a v15 da pesquisa documental e moveu relacionamentos para v16; CI oficial verde na PR #491."
agent: castor
timestamp: "2026-11-08T23:09:52Z"
tags: [sqlite, rust, anti-ies, bdd, merge, ci]
---

# Para lembrar no próximo ciclo

**Fato:** na [PR #491](https://github.com/franklinbaldo/ies/pull/491), a `main` ganhou `0015_pesquisa_documento_paginas.sql` (#513), colidindo com a antiga v15 de relacionamentos. Fiz [merge verdadeiro com dois pais](https://github.com/franklinbaldo/ies/commit/9dedb29e84ddc94628d4d0700e970c8635b6dc8d) e declarei relacionamentos como **v16**, sem descartar a pesquisa v15 nem mudanças paralelas em `processo_envio.rs`. [CI oficial #42533498988](https://github.com/franklinbaldo/ies/actions/runs/42533498988) **7/7 PASS** no SHA final, branch `behind=0`, PR Ready/mergeável. [Handoff](https://github.com/franklinbaldo/ies/pull/491#issuecomment-6728465543).

**Retomar:** aguardar integração da #491 por 🐘/🦡, sem reimplementar BDD-258-01. BDD-258-02/03/04 seguem na [issue #258](https://github.com/franklinbaldo/ies/issues/258). Compatibilidade IES↔anti-IES **sintética PASS**; dois cards PEN1 (53 interações, hashes conferidos antes), confronto semântico independente **NOT_RUN** e live em treinamento **NOT_RUN**. Esta rodada não gerou um novo RED.

No [Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6728186890), a ideia de procurar o contraplano/negativo e não ampliar a mesma imagem é uma analogia útil de **observação independente**. Inspiração para a pesquisa, não prova do comportamento do SEI.
