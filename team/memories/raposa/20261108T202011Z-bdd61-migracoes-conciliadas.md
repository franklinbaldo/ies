---
type: AgentMemory
title: "Busca documental BDD-61-03: v14 reconciliada e validada"
description: "PR #489 sincronizada em merge de dois pais após v12/v13; CI 7/7 do SHA final e pronta para integração."
agent: raposa
timestamp: "2026-11-08T20:20:11Z"
tags: [cli, bdd, anti-ies, sqlite, integracao]
---

# Retomar

Fato: a [PR #489](https://github.com/franklinbaldo/ies/pull/489) recebeu o
[merge verdadeiro `aedf5c5`](https://github.com/franklinbaldo/ies/commit/aedf5c56723734c2412dc12d9ef250629ad83136)
depois da integração da #487: esquema SQLite `0012 → 0013 → 0014`,
`user_version=14`, sem perder navegação do Controle ou cancelamento.
[CI rápido](https://github.com/franklinbaldo/ies/actions/runs/42519305237) PASS;
[CI completo](https://github.com/franklinbaldo/ies/actions/runs/42519312184)
7/7 PASS no head final, inclusive os testes de entrada inválida que aguardavam execução.

Retomar após revisão e integração do 🐘/🦡; então avançar os BDD-61-01/02
sem reimplementar o BDD-61-03. Os [cards GO/UnB](https://github.com/franklinbaldo/ies/issues/61#issuecomment-6726610574)
foram consultados; hashes conferidos em inspeção anterior, mas
confronto independente de HTML e live permanecem não executados.
A [conversa do Cafezinho](https://github.com/franklinbaldo/ies/issues/388)
lembra que duas histórias podem produzir o mesmo resultado: não confundir
concordância dos simuladores com observação real.
