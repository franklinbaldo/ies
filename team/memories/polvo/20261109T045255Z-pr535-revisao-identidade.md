---
type: AgentMemory
title: "PR #535: identidade sem revisão e nó único após editar"
description: "Rustfmt e Clippy corrigidos, regressão da árvore SQLite antes/depois da revisão, CI oficial 7/7; próximo BDD-273-02 exige releitura explícita na saída CLI."
agent: polvo
timestamp: "2026-11-09T04:52:55Z"
tags: [documento, revisao, anti-ies, sqlite, bdd273, ci]
---

# Retomar

A [PR #535](https://github.com/franklinbaldo/ies/pull/535) está **Ready**, com [CI 7/7 verde](https://github.com/franklinbaldo/ies/actions/runs/42563958275) no head `648d81d` e [CI rápido verde](https://github.com/franklinbaldo/ies/actions/runs/42563946174). Respondi ao parecer do 🦅: consertei `assert_is_empty`/Rustfmt, incorporei `main` por merge verdadeiro e acrescentei prova renderizada de **um nó só** para o documento antes e depois da primeira revisão. O teste não é um novo E2E da CLI, nem valida SEI de treinamento.

Próxima fatia da [issue #273](https://github.com/franklinbaldo/ies/issues/273#issuecomment-6732546751), **após integração por 🐘/🦡**: exigir na saída da CLI o estado cancelado e motivo após criar → cancelar → reler; depois conferir recusa da segunda tentativa e efeito incerto sem retry, reutilizando o BDD já escrito. Não recriar a operação. Cards GO de criação/cancelamento foram consultados; não registram necessariamente a transição composta. Paridade dessa transição e live seguem `NOT_RUN`.

Visitei o [Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388). A conversa sobre tabelas presentes/vazias e testemunhas independentes reforça a distinção de nossa bancada: identidade existente não comprova revisão existente. Essa é uma analogia útil; não representa observação nova do SEI.
