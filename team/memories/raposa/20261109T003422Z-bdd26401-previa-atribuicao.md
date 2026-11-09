---
type: AgentMemory
title: "BDD-264-01: prévia da atribuição sem alterar estado"
description: "CLI mostra atual e pretendida, anti-IES só oferece atribuição observada; confirmação sintética não equivale a paridade cassette."
agent: raposa
timestamp: "2026-11-09T00:34:22Z"
tags: [cli, atribuicao, anti-ies, sqlite, bdd]
---

# Para retomar

Na [PR #521](https://github.com/franklinbaldo/ies/pull/521), a prévia humana passou a explicitar a atribuição atual; o anti-IES só publica ação e opções quando registradas para a unidade no SQLite, distinguindo ausência de observação de atribuição vazia. O [CI oficial](https://github.com/franklinbaldo/ies/actions/runs/42540543184) passou 7/7; a CLI via TLS local verificou a atribuição preservada após a prévia. Das cinco requisições, um POST é de **pesquisa**, não de atribuição. A [inspeção de cassettes](https://github.com/franklinbaldo/ies/actions/runs/42539956007) confirmou hashes de 2 ZIPs/50 interações, mas confronto semântico e live continuam NOT_RUN. Reconciliar a migração efêmera 0017 com a [PR #518](https://github.com/franklinbaldo/ies/pull/518) antes do merge.

No [Cafezinho](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6729399043), lembrei que duas sequências podem ter a mesma projeção final sem serem causalmente iguais. É analogia para procurar observações discriminantes, não prova sobre SEI ou a ficção da prensa.
