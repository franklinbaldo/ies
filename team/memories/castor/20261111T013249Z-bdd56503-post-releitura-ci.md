---
type: AgentMemory
title: "BDD-565-03: POST local único, releitura real e RED/GREEN comprovados"
description: "PR #736 conclui a vertical de confirmação das colunas com transição tipada no SQLite v38; CI oficial 7/7 no head final, cassette PEN1 sem ação de salvar e live não executado."
agent: castor
timestamp: "2026-11-11T01:32:49Z"
tags: [ies, castor, bdd, colunas, anti-ies, sqlite, red-green, ci]
---

# Retomada da oficina

**Fato:** depois de integrar a #711, assumi BDD-565-03 previamente escrito pelo 🐘 na [#565](https://github.com/franklinbaldo/ies/issues/565). Abri [PR #736](https://github.com/franklinbaldo/ies/pull/736), com teste do binário IES real via TLS local → anti-IES → SQLite v38 → segunda CLI. O [RED oficial #42937411342](https://github.com/franklinbaldo/ies/actions/runs/42937411342) mostrou HTTP 405 no único POST, seguido de releitura que manteve marcas anteriores (recusa correta). Implementei rota POST e atualização transacional tipada sem migração nova. Após corrigir `clippy::manual_let_else`, o [CI oficial #42938914957](https://github.com/franklinbaldo/ies/actions/runs/42938914957) fechou **7/7 PASS** no head `753f67c3`: cinco GETs, um POST, depois igual ao alvo e consulta independente coerente. Branch incorporou a main por merge verdadeiro `d53a663`; havia somente memória OKF disjunta posterior.

**Limites:** [card PEN1](https://github.com/franklinbaldo/ies/blob/main/cassettes/pen1/unidade-painel-controle-pen1--5a4b1d4eb87d.md) teve hashes/estrutura conferidos em [Actions #42919876483](https://github.com/franklinbaldo/ies/actions/runs/42919876483), mas registra dois GETs de outras ações. IES↔anti-IES↔SQLite **PASS sintético**; confronto semântico bilateral de escrita com cassette **NOT_RUN**; treinamento/live **NOT_RUN**. Não inventar equivalência com o SEI, nem repetir a escrita PEN1.

**CONTINUAR:** revisão 🦅 e integração 🐘/🦡 da #736; BDD-565-04 só na fila apropriada, considerando a nova prioridade do proprietário em [#716](https://github.com/franklinbaldo/ies/issues/716) e o AGENTS.md recém-atualizado. Visitei [Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388) e li a conversa sobre observação parcial: a falta de um item numa lista incompleta não prova ausência. Analogia metodológica, não evidência de colunas.
