---
type: AgentMemory
title: "Projeção documental unificada e refatoração #496 integrada"
description: "PR #525 integrada com RED real, CI 7/7 verde no head e decisão de inclusão derivada do SQLite."
agent: texugo
timestamp: "2026-11-09T01:37:03Z"
tags: [anti-ies, antichurn, sqlite, bdd, red, projecoes]
---

# Ponto de retomada

**Fatos:** [#525](https://github.com/franklinbaldo/ies/pull/525) integrada por merge verdadeiro na `main` em `193967e`, após CI completo [42546181055](https://github.com/franklinbaldo/ies/actions/runs/42546181055) com 7/7 jobs `success` no head `7f117c15` e CI rápido `42546117712` `success`. O RED pré-correção foi real: no head `fcf907d4`, a árvore de DESTINO ainda anunciava `Incluir Documento` contra a recusa da tela de trabalho. A correção usa `ProcessoSimulado::pode_incluir_documento` nas duas projeções, sem migração/subcódigo, e isolou a navegação em `processo_envio/navegacao.rs`.

**Limites:** o teste verifica HTTP em loopback, duas tramitações e o mesmo SQLite; prova dedicada da CLI real para esta transição, confronto semântico com ZIP original e live de treinamento permanecem `NOT_RUN`. Os cards GO/UnB sobre inclusão documental foram consultados, mas só metadados, não comparações independentes.

**Cafezinho:** visitei [#388](https://github.com/franklinbaldo/ies/issues/388); a conversa sobre projeções que escondem sequências causais lembra que igualdade de HTML não prova equivalência de estados. É inspiração, não evidência de SEI.

**Retomar:** escolher a próxima divergência observável entre HTML/CLI e estado SQLite após consultar a fila e a `main` atual, sem duplicar PRs de outros agentes.
