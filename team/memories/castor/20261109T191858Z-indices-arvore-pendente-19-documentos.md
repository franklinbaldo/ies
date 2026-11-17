---
type: AgentMemory
title: "Índices de nós não podem apagar documentos observados"
description: "PR #598 integrada com E2E sintético de 19 documentos visíveis e pasta pendente, CI 7/7; falta provar a forma exata 19/20 e confrontos externos."
agent: castor
timestamp: "2026-11-09T19:18:58Z"
tags: [anti-ies, arvore, bdd, sqlite, cassettes, prova]
---

# Lembrar na próxima rodada

**Fato:** a [PR #598](https://github.com/franklinbaldo/ies/pull/598) foi integrada na `main` em [44c055c](https://github.com/franklinbaldo/ies/commit/44c055cfb377114987c4e467287b81d8763a1b30), após merge verdadeiro com a `main`. O [CI #42681418855](https://github.com/franklinbaldo/ies/actions/runs/42681418855) passou 7/7 no head `4f0a801b`. A projeção agora coloca uma pasta não carregada depois dos 19 nós documentais sem sobrescrevê-los. A CLI publica apenas metadados comprovados com aviso de parcialidade.

**Limite:** simulação 19 documentos mais pasta pendente **não é** reprodução de 19/20 no interior de uma mesma pasta; confronto semântico da falha com cassette e treinamento são `NOT_RUN`. Integridade anterior de 498 interações UnB não altera isso. O teste foi comitado antes da correção, mas não houve execução completa de RED no SHA anterior.

**Retomada:** 🐘 declarou BDD-342-01/02 completos sinteticamente; ensaios restantes estão na [#583](https://github.com/franklinbaldo/ies/issues/583). Escolher outra issue livre com BDD já escrito, branch nova, sem continuar #598.

**Cafezinho:** [comentário sobre a gaveta com índice livre](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6749113275). Analogia de carpintaria, não observação do SEI.
