---
type: AgentMemory
title: "Blocos: CI reconciliado e corrida de memórias na integração"
description: "PR #597 manteve 7/7 após merge verdadeiro 3fab873; novas memórias da main reabriram behind sem alterar o código; o próximo BDD precisa observar POST de paginação."
agent: polvo
timestamp: "2026-11-09T20:54:54Z"
tags: [blocos, bdd256, anti-ies, ci, integracao, cassettes]
---

# Retomada

- **Fato:** [PR #597](https://github.com/franklinbaldo/ies/pull/597) seguiu Ready, BDD-256-01 sem novo código. Merge de dois pais [`3fab8734`](https://github.com/franklinbaldo/ies/commit/3fab87343b8ba54a11b899b5111214c204012f49) incorporou cinco memórias OKF da `main` em arquivos disjuntos: `behind=0` no instante do merge. O [CI rápido #42696655611](https://github.com/franklinbaldo/ies/actions/runs/42696655611) e [CI completo #42696723917](https://github.com/franklinbaldo/ies/actions/runs/42696723917) passaram, **7/7**, no head `3fab8734`. A `main` avançou depois com outras duas memórias, portanto novo `behind=2`; há [handoff na PR](https://github.com/franklinbaldo/ies/pull/597#issuecomment-6751212892). Não confundir CI de head anterior com gate de um futuro merge.
- **Provas:** CLI real ↔ anti-IES ↔ SQLite PASS sintético do BDD-256-01; inspeção UnB/GO (324 interações) verificou integridade, **não** semântica independente, que permanece NOT_RUN; treinamento NOT_RUN, nenhum pedido ou escrita. Não duplicar testes nem alterar branch alheia.
- **Próxima fatia, somente depois de integração:** BDD-256-02 da [issue #256](https://github.com/franklinbaldo/ies/issues/256): `ies bloco listar --tipo interno --pagina 2`. O [card GO de bloco interno](https://github.com/franklinbaldo/ies/blob/main/cassettes/go/bloco-interno-ciclo-go--db0d47ba2ff4.md) documenta 57 requisições, incluindo 29 POST, **sem demonstrar ainda qual POST muda de página**. Observar corpo original no Actions antes de modelar a transição, e não confundir `total_declarado` com conteúdo de página não observada. A indisponibilidade de DNS no sandbox foi verificada; não inventar a requisição.
- **Cafezinho:** visitei [#388](https://github.com/franklinbaldo/ies/issues/388). A conversa distingue leitura e autorização de acesso a uma versão; serve de lembrete para não tomar uma página do índice pela própria gaveta documental, não por evidência do SEI.
