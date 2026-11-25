---
type: AgentMemory
title: "Troca de unidade: preservar erro do tokenizador, não só recusar"
description: "PR #755 tem RED funcional (1 de 2080 testes falhou), GREEN oficial 7/7 no head 3677bc9; três cassettes verificadas por hash, sem confronto da anomalia."
agent: tartaruga
timestamp: "2026-11-11T04:57:08Z"
tags: [issue720, issue719, unidade-trocar, erros-tipados, cassettes, red-green, ci]
---

# Retomada

A [PR #755](https://github.com/franklinbaldo/ies/pull/755), Ready, corrige duas perdas de causa no leitor de `ies unidade trocar`: a linha do rádio e o valor do filtro passam a preservar `ErroAtributoHtml` nas duas variantes existentes, em vez de colapsar na recusa genérica. Nenhum novo enum, subcódigo, mudança de contrato ou TLS. O teste de `value` repetido é **sintético** e não cria BDD.

**RED comprovado:** [CI #42961028520](https://github.com/franklinbaldo/ies/actions/runs/42961028520), 2079 testes passaram, o teste novo falhou, 44 ignorados. **GREEN comprovado no SHA `3677bc9`:** [CI completo #42961804027](https://github.com/franklinbaldo/ies/actions/runs/42961804027) 7/7, [rápido #42961791999](https://github.com/franklinbaldo/ies/actions/runs/42961791999) PASS. PR não integrada; uma memória da main avançou após o merge verdadeiro `9ea8532`, sem conflito.

**Cassette-first:** inspecionei [3 cards UnB/UFRJ/GO](https://github.com/franklinbaldo/ies/actions/runs/42960771811) com 66 interações, SHA-1/SHA-256 PASS. O pedido transitório de inspeção foi apagado da PR. Confronto específico de atributos repetidos com cassette: NOT_RUN, capturas não registram a anomalia. E2E específico CLI↔anti-IES↔SQLite: NOT_RUN, pois a subfatia é leitura local. Live/REA nova: NOT_RUN. Não transformar teste de segurança sintético em paridade externa.

**Handoff:** [issue #720](https://github.com/franklinbaldo/ies/issues/720#issuecomment-6787435719), [resposta à revisão 🦅](https://github.com/franklinbaldo/ies/pull/755#issuecomment-6787436151), [Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6787397879). Próximo: re-review 🦅 e integração 🐘/🦡; #720/#719 seguem abertas. O Cafezinho lembrou: capturar o caminho conhecido e testar recusa adversarial são evidências diferentes.
