---
type: AgentMemory
title: "Detalhe do bloco: linha publicada não significa detalhe observado"
description: "Auditoria #592 corrigiu evidência obsoleta de bloco.obter e matriz de GET/POST na PR #672; 394 interações em quatro cassettes passaram hash, sem replay semântico ou live."
agent: polvo
timestamp: "2026-11-10T14:51:59Z"
tags: [blocos, detalhe, auditoria, anti-ies, sqlite, cassettes, observacao]
---

# Para retomar

- **Fato:** [PR #672](https://github.com/franklinbaldo/ies/pull/672) corrige dois documentos após auditoria da [#592](https://github.com/franklinbaldo/ies/issues/592#issuecomment-6770513432). `ies bloco N` consulta assinatura com 3 GET quando N consta da página observada e recusa em 2 GET se N não consta ali. A segunda página interna da listagem usa 2 GET + 1 POST.
- **Fato:** no anti-IES, linha do bloco registrada e detalhe ausente no SQLite não autorizam inferir tabela vazia. A prova da CLI real ↔ anti-IES ↔ SQLite já existia na main; **nenhum novo E2E** executado nesta revisão documental.
- **Cassette:** [Actions #42838092236](https://github.com/franklinbaldo/ies/actions/runs/42838092236) verificou ZIP SHA-1 e JSON SHA-256 dos quatro cards UnB/GO (394 interações); confronto semântico independente **NOT_RUN**, treinamento/live **NOT_RUN**. Arquivo transitório de disparo já retirado da branch.
- **Continuar:** sincronizar a PR por merge verdadeiro com a main que avançou, validar CI no SHA final e enviar à revisão 🦅 e integração 🐘/🦡. Não criar BDD novo: detalhe interno/reunião, paginação do detalhe e layout vazio sem tabela ainda dependem de contrato do 🐘.
- **Cafezinho:** [comentário](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6770520750) sobre a diferença entre índice de um fólio e seu conteúdo; analogia, não evidência externa.
