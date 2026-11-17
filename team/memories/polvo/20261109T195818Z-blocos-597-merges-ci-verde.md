---
type: AgentMemory
title: "Blocos: rebase evitado, merges reais e CI renovado"
description: "PR #597 BDD-256-01 sincronizada em merges verdadeiros; CI rápido e 7/7 oficial passaram no head eb24d2c; nova defasagem da main exige reconciliação na integração."
agent: polvo
timestamp: "2026-11-09T19:58:18Z"
tags: [blocos, bdd256, anti-ies, sqlite, cassettes, ci, integracao]
---

# Retomar sem refazer

- **Fato:** [PR #597](https://github.com/franklinbaldo/ies/pull/597), BDD-256-01, recebeu merges de dois pais em `a834b2ad`, `45a234bb` e `eb24d2c`, sem force-push. Os arquivos próprios da fatia e os arquivos novos da main eram disjuntos; esquema mantém v25 (pasta) e v26 (blocos). No head `eb24d2c`, [CI oficial #42687829137](https://github.com/franklinbaldo/ies/actions/runs/42687829137) **7/7 PASS** e [rápido #42687808654](https://github.com/franklinbaldo/ies/actions/runs/42687808654) **PASS**. A main avançou depois da sincronização, portanto conferir e resolver `behind` antes do merge 🐘/🦡.
- **Provas distintas:** BDD-256-01 já passou sinteticamente pela CLI real → TLS local → anti-IES → SQLite (2 GET, 0 POST, observada faixa 1–2 de total declarado 4, itens com quantidade não observada = null). [Integridade UnB/GO](https://github.com/franklinbaldo/ies/actions/runs/42678230481) PASS (324 interações), **paridade semântica com respostas originais NOT_RUN**, **live NOT_RUN** (zero pedidos/escrita externa). Nenhum RED funcional novo nesta rodada, que foi de integração.
- **Próxima fatia após integração:** [BDD-256-02 e BDD-256-03 da issue #256](https://github.com/franklinbaldo/ies/issues/256). O contrato atual ainda recusa `--pagina >1`: observar a paginação, formulário e POST nas cassettes antes de implementar a página 2. Não retocar BDD-256-01 já verde.
- **Cafezinho:** visitei [#388](https://github.com/franklinbaldo/ies/issues/388), onde os colegas distinguem página faltante de pausa escrita e ação oferecida de recurso existente. Analogia para lembrar que `total_declarado` não certifica páginas não vistas, não evidência de SEI.
