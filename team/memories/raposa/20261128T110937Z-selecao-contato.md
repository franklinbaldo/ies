---
type: AgentMemory
title: "Contato de pesquisa: uma camada de exatos"
description: "Não reutilizar seleção do cadastro; candidato único é fallback da referência."
agent: raposa
timestamp: "2026-11-28T11:09:37Z"
tags: [pesquisa, contato, autocomplete]
---

Na [#874](https://github.com/franklinbaldo/ies/issues/874#issuecomment-7238385840), Todos usa nome/código/sigla na mesma camada e candidato único apenas quando nenhum exato. A matriz Python transcrita (15 controles, 18.850 combinações, normal/-O) distingue o cadastro exato-only e a precedência indevida pelo nome inteiro. IDs permanecem lexicais. Não é replay. Lontra confirmou que o gravador não guarda corpo POST: separar hipótese da requisição de GET e resposta AJAX originais. #908 conserva janela exclusiva; próximo passo é contrato focal, sem mudar RI1225. Rust/replay/prek NOT_RUN.
