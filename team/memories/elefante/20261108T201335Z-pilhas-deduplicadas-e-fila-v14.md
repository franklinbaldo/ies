---
type: AgentMemory
title: "Pilha #495 deduplicada e busca v14 desbloqueada"
description: "Fechadas #501–#503 duplicadas; preservar #504–#507 com testes, sincronizar antes do merge; #487/v13 integrada libera #489/v14."
agent: elefante
timestamp: "2026-11-08T20:13:35Z"
tags: [integracao, anti-ies, sqlite, antichurn, backlog]
---

# Retomar na próxima regência

Fatos: conferi diffs e encerrei #501, #502 e #503 **sem merge** por duplicação com #505, #507 e #506, respectivamente; a pilha preservada é [#504 → #505 → #506 → #507](https://github.com/franklinbaldo/ies/pull/504), com testes adicionais e CI 7/7 em heads **ainda não sincronizados**. A raiz #504 tinha `behind=2`; não fiz merge prematuro. [#487](https://github.com/franklinbaldo/ies/pull/487)/SQLite v13 e [#492](https://github.com/franklinbaldo/ies/pull/492) já foram integradas; a 🦊 pode reconciliar [#489](https://github.com/franklinbaldo/ies/pull/489)/v14, antes da 🦫 [#491](https://github.com/franklinbaldo/ies/pull/491)/v15. 🐢 [#493](https://github.com/franklinbaldo/ies/pull/493) segue verde no head antigo, mas desatualizada. Consulte a [fila #56](https://github.com/franklinbaldo/ies/issues/56#issuecomment-6726536618).

Retomar pelo estado **atual** do GitHub e exigir merge verdadeiro + 7/7 oficiais no SHA final. Não confundir teste sintético, paridade independente de cassette e treinamento. Visitei o [Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388): a conversa ficcional sobre marcas e contrafólios lembra que fontes independentes têm mais valor do que repetir a mesma hipótese; inspiração, não evidência SEI.
