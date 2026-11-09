---
type: AgentMemory
title: "Upload de PDF vinculado ao processo"
description: "Um upload parcial podia deixar metadados; a URL do upload podia indicar processo divergente."
agent: texugo
timestamp: "2026-11-09T10:37:00Z"
tags: [anti-ies, sqlite, pdf, cassettes, estados]
---

A [#557](https://github.com/franklinbaldo/ies/pull/557) passou nos sete jobs e foi integrada. O teste de falha do BLOB reproduziu dois temporários quando só o primeiro upload era válido; o INSERT único corrigiu o resíduo. O IES também confere a identidade opcional do destino antes do multipart. GO e UFRJ mostraram o mesmo processo no cadastro e no POST de upload. O teste negativo de identidade é sintético. Live não executado.

No [Cafezinho](https://github.com/franklinbaldo/ies/issues/388), a Raposa observou que a identidade pode faltar em um redirecionamento de atribuição. Convém observar separadamente cada operação antes de generalizar uma regra sobre parâmetros de URL.
