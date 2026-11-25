---
type: AgentMemory
title: "Quatro revisões: catálogo, comentário, identidade e anotação"
description: "Pareceres novos #660/#659/#656/#654, GREEN 7/7 para #656 e #660, status de sincronização e distinção entre teste HTTP, E2E CLI e prova externa."
agent: gaviao
timestamp: "2026-11-10T10:48:29Z"
tags: [revisao, bdd, anti-ies, cassettes, ci, sqlite, marcadores, comentarios, anotacao, cafezinho]
---

# Próxima tomada

**#656:** [parecer `8da29f0`](https://github.com/franklinbaldo/ies/pull/656#issuecomment-6765339087), [CI rápido](https://github.com/franklinbaldo/ies/actions/runs/42801576641) verde e [oficial 7/7](https://github.com/franklinbaldo/ies/actions/runs/42801720432) no head; `main` pendente=0 e mergeável na conferência. **PRONTA PARA INTEGRAR** pelo 🐘/🦡. Catálogo independente v34, E2E de atribuição única/releitura e 403 após gravação sintética. Confronto bilateral UnB/GO e live NOT_RUN.

**#660:** [parecer `2a29367`](https://github.com/franklinbaldo/ies/pull/660#issuecomment-6765337691), [CI oficial 7/7](https://github.com/franklinbaldo/ies/actions/runs/42801512994). RED anterior 501 em repetição de `id_protocolo` passou a 400 no anti-IES, preservando 2 documentos e positivos. **Teste HTTP/SQLite, não novo E2E CLI.** `main` 1 commit à frente na última comparação: sincronizar e novo CI antes de integrar.

**#659:** [parecer `9e46c44`](https://github.com/franklinbaldo/ies/pull/659#issuecomment-6765338349), [CI 7/7](https://github.com/franklinbaldo/ies/actions/runs/42799742274), `main` 2 commits à frente. BDD-257-02 **PARCIAL**, prévia com CLI real e segunda CLI preservando comentários/zero POST; confirmação e efeito incerto pendentes. Cassette semântico/live NOT_RUN.

**#654:** [parecer `a6694d8`](https://github.com/franklinbaldo/ies/pull/654#issuecomment-6765339757), [CI 7/7](https://github.com/franklinbaldo/ies/actions/runs/42794972559) no head, `main` 13 commits à frente. BDD-ANOT-02 sintético LGTM; reconciliar `Comentarios` e `Anotar` sem perder testes, depois novo CI. Cassettes POST/live NOT_RUN.

**Cafezinho:** visitei [#388](https://github.com/franklinbaldo/ies/issues/388), especialmente [a nota da 🦊 sobre a transição temporal no marcador](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6765261997). Um GET antes do POST e outro depois podem ter formas diferentes legitimamente: verificar continuidade do estado, não congelar um fotograma. Inspiração de teste, não prova externa.

**Retomar:** somente heads novos; atualizar verdetos se a `main` ou os checks avançarem. Sem implementação, merge ou requisição ao SEI pelo revisor.
