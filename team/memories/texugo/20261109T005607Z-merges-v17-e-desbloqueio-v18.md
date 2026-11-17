---
type: AgentMemory
title: "Três merges verdes; atribuição aguarda v18 e cancelamento tem RED real"
description: "Integradas #518, #520 e #522 com head 7/7; #521 precisa preservar v17 e ir a v18; #523 tem falha concreta na releitura do motivo opaco."
agent: texugo
timestamp: "2026-11-09T00:56:07.060Z"
tags: [anti-ies, antichurn, integracao, sqlite, bdd, cassettes]
---

# Próxima rodada

**Fatos verificados:** integrei com merge verdadeiro e 7/7 gates oficiais verdes no head [#518](https://github.com/franklinbaldo/ies/pull/518) (`f77aa60b`, prévia de criar processo e SQLite **v17**), [#520](https://github.com/franklinbaldo/ies/pull/520) (`a98145ba`, identidade/preservação de relacionamentos na releitura) e [#522](https://github.com/franklinbaldo/ies/pull/522) (`a4fad83d`, CLI recusa documento sem src e oculta somente o inacessível). Sem CI pós-merge; estes testes são sintéticos, não equivalência independente aos cassettes ou live.

**Destravar sem refazer capacidade:** a [#521](https://github.com/franklinbaldo/ies/pull/521#issuecomment-6729666635) ainda chama sua migração `0017_processo_atribuicao.sql`; após #518, conciliar `main` por merge, deixar criação em v17 e mover atribuição para **v18**, revalidando CI. A [#523](https://github.com/franklinbaldo/ies/pull/523#issuecomment-6729666975) falhou de fato no novo BDD de motivo opaco: código 3 na confirmação com Unicode/newline/ampersand; diagnóstico de stdout + fronteira de codificação/releitura, sem repetir POST incerto, manter GREEN como requisito.

Visitei o [Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388): a reflexão de que uma projeção comum não identifica a sequência causal é inspiração; não evidência externa do SEI. Os resultados de integridade de cards não são confronto semântico.
