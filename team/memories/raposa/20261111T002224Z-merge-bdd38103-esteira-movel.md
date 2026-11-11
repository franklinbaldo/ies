---
type: AgentMemory
title: "BDD-381-03: merge verificável mesmo com main muito móvel"
description: "PR #708 reconciliada por merge verdadeiro e CI 7/7 no head 77b16c1f; a main continuou avançando sem alterar os quatro arquivos da fatia. Re-review e integração pendentes."
agent: raposa
timestamp: "2026-11-11T00:22:24Z"
tags: [ux, bloco, bdd-381-03, git, ci, anti-ies, sqlite, cassette-first]
---

# Retomar sem repetição de trabalho

**Fato:** a [PR #708](https://github.com/franklinbaldo/ies/pull/708) mantém a correção A/B: pertencimento é relativo ao bloco pedido, nunca exclusividade global. Reconciliada duas vezes com a `main` por merges reais de dois pais, sem force-push, a árvore gerada via GitHub foi conferida contra o SHA da árvore do merge local. O head `77b16c1f` passou [CI oficial #42930293240](https://github.com/franklinbaldo/ies/actions/runs/42930293240) **7/7** e [rápido #42930286812](https://github.com/franklinbaldo/ies/actions/runs/42930286812) **PASS**.

**Estado móvel:** durante esses gates a `main` avançou novamente por integrações paralelas, deixando a branch 12 commits atrás no último confronto, embora mergeável. Não repetir a implementação nem atribuir testes externos que não rodaram; re-review 🦅 e integração 🐘/🦡 com reconciliação apenas quando necessária. [Handoff #381](https://github.com/franklinbaldo/ies/issues/381#issuecomment-6783122269).

**Provas:** CLI real↔anti-IES↔SQLite **PASS sintético**; cassette UnB sem prova semântica da transição A/B e treinamento **NOT_RUN**. O [Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388) reiterou uma distinção útil: estado observado de um item não equivale a oferta global de uma ação.

**Próximo BDD:** após integrar 03, iniciar 381-04 somente com orientação do 🐘 para a leitura pública independente do detalhe de bloco interno/reunião ([nota 🐙](https://github.com/franklinbaldo/ies/issues/381#issuecomment-6782489816)). Não inventar nova flag.
