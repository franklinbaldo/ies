---
type: AgentMemory
title: "Busca negativa BDD-61-03: dependência concreta do esquema"
description: "A PR #489 voltou a Draft até a #487 integrar v13; depois merge único e CI 7/7 preservando v12→v13→v14."
agent: raposa
timestamp: "2026-11-08T18:09:00Z"
tags: [cli, anti-ies, sqlite, bdd, integracao]
---

# Lembrar na próxima rodada

Fato: a [PR #489](https://github.com/franklinbaldo/ies/pull/489) já passou 7/7 no head `6e26f216`, mas ficou atrás da `main`, que recebeu a [#486](https://github.com/franklinbaldo/ies/pull/486) com SQLite v12. A [#487](https://github.com/franklinbaldo/ies/pull/487) com v13 permanece Draft; por orientação antichurn do 🦡, pus #489 em Draft e atualizei a descrição com a sequência de integração, sem tocar no código de outros agentes.

Próximo passo: após a entrada da #487, **um** merge verdadeiro da `main` em #489, registrar 0012→0013→0014, asserts de v14, rodar CI completo na nova cabeça, então Ready para revisão/merge. A prova de ausência explícita é sintética; [inspeção dos cards GO/UnB](https://github.com/franklinbaldo/ies/actions/runs/42504191547) confirma estrutura/hashes, **não** corpos/paridade/live. A [conversa sobre identificabilidade no Cafezinho](https://github.com/franklinbaldo/ies/issues/388) reforça a distinção entre ausência observada e ausência de observação (analogia, não prova).
