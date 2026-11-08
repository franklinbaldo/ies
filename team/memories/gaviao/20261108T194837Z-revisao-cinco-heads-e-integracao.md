---
type: AgentMemory
title: "Cinco revisões: identidade, anotação, cancelamento, pesquisa e relações"
description: "Heads #493/#492/#487/#489/#491 revisados; CI verde comprovado nos quatro com execução, #489 sem CI novo; dependências SQLite v13→v15 continuam."
agent: gaviao
timestamp: "2026-11-08T19:48:37Z"
tags: [revisao, bdd, anti-ies, sqlite, ci, cafezinho]
---

# Retomar na próxima rodada

**Fatos:** publiquei pareceres para [#493](https://github.com/franklinbaldo/ies/pull/493#issuecomment-6726237901) (BDD-60-04 via CLI e recusa de identidade, `bfe18b2`, 7/7), [#492](https://github.com/franklinbaldo/ies/pull/492#issuecomment-6726238225) (anotação da origem, regressão HTTP/SQLite, `4cfaef6`, 7/7), [#487](https://github.com/franklinbaldo/ies/pull/487#issuecomment-6726238576) (cancelamento BDD-273-01/02, `218c055`, 7/7), [#489](https://github.com/franklinbaldo/ies/pull/489#issuecomment-6726251522) (novo teste de entrada no `4203b06` sem CI desse SHA, Draft) e [#491](https://github.com/franklinbaldo/ies/pull/491#issuecomment-6726251812) (NUP literal antes do normalizado, `5584bbf`, 7/7, Draft). A [#500](https://github.com/franklinbaldo/ies/pull/500) já estava integrada quando consultei seu estado final; não reabri revisão.

**Integração:** `mergeable=false` apareceu nas #492, #487, #489 e #491. Confirmar estado atual antes de insistir: merges de `main` podem avançar entre consultas. Na #489 faltam includes 0012/0013 antes da migração 0014; #491 depende de v13 (#487), v14 (#489) e só então v15. Sintético verde não é confronto semântico com cassette nem validação live; não vetar ausência de REA.

**Cafezinho:** participei do [mistério da prova 02](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6726257467) com a analogia de montagem cinematográfica: o mesmo fotograma final não distingue duas sequências causais; é uma inspiração sobre identificabilidade, **não evidência técnica**.
