---
type: AgentMemory
title: "Recusa de histórico cancelado integrada; blocos voltam à fila e andamento v27 segue atrás da main"
description: "PR #604 integrada com CI 7/7; BDD-256-01 reconciliado; #602 depende de sincronização apesar do CI verde; #526 ainda conflitante."
agent: elefante
timestamp: "2026-11-09T21:10:53Z"
tags: [integracao, fila, bdd, sqlite, anti-ies, cassettes, treinamento, cafezinho]
---

# Fatos para retomar

- **Main observada:** `f81933987c3ed1c91194a8ec611b161596ac1d46`. Integrei [#604](https://github.com/franklinbaldo/ies/pull/604) por merge verdadeiro de dois pais nesse SHA, head `b7dfd3f`, `behind=0`, [CI completo #42698921520](https://github.com/franklinbaldo/ies/actions/runs/42698921520) **7/7 success**. BDD-60-05 cobre **mais uma recusa sintética**: histórico anunciado na árvore, mas documento cancelado antes do GET; CLI recusa HTTP 403/saída 1, sem fallback, GET do corpo ou POST. **#60 ainda PARCIAL**; confronto da corrida nos cassettes NOT_RUN, treinamento NOT_RUN. Handoff na [#60](https://github.com/franklinbaldo/ies/issues/60#issuecomment-6751535194).
- **Blocos:** [#597](https://github.com/franklinbaldo/ies/pull/597) já integrada, SQLite **v26**, `BDD-256-01` marcado PASS **sintético** no corpo da [#256](https://github.com/franklinbaldo/ies/issues/256). Removi etiqueta `assumida` e avatar do título: **BDD-256-02 paginação** e **BDD-256-03 detalhe** livres, já especificados, começar por cards/cassettes e prova CLI real↔anti-IES↔SQLite. Sem confronto semântico ou live novos.
- **Andamento:** [#602](https://github.com/franklinbaldo/ies/pull/602) usa `0027_andamento_processo.sql`, head `e6a9c73` com [CI oficial #42699121943](https://github.com/franklinbaldo/ies/actions/runs/42699121943) **7/7**, mas `behind=18` após merges; remover disparador temporário de cassettes, merge verdadeiro da main, novo 7/7 e só então integrar. BDD-259-01 PASS sintético na branch, não na main. [Orientação](https://github.com/franklinbaldo/ies/issues/259#issuecomment-6751536039).
- **Criação:** [#526](https://github.com/franklinbaldo/ies/pull/526) Draft e conflitante; a migração de criação v18 antiga deve ser reconciliada com v26 integrada e v27 proposta; **v28 condicional** se #602/v27 integrar primeiro. Preservar E2E das duas CLIs e único POST.
- **PDF externo:** [#601](https://github.com/franklinbaldo/ies/pull/601) integrada. Live UnB comprovou que PDF externo `0012272` foi criado e recuperado com hash igual, mas CLI retornou saída 3 errada. Correção passou offline/sintética, novo live corrigido **NOT_RUN**. [#583](https://github.com/franklinbaldo/ies/issues/583#issuecomment-6751536718), sem repetir POST nem liberar confirmação pública.
- [Fila #56](https://github.com/franklinbaldo/ies/issues/56#issuecomment-6751537501) atualizada. O radar semanal do Todos para 30/10–06/11 já constava na #56, sem retrabalho. Consultei o [Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388): a conversa sobre gravação e legenda separa o efeito real no SEI do estado declarado pela CLI; é analogia, não teste.

# Retomar

Conferir `main`, SHAs, PRs #602/#526, CI `pull_request` de 7/7 no head sincronizado, e priorizar fatias livres sem reserva antecipada. Cassette-first, nunca cassette-gated; provas sintética, cassette e treinamento são independentes. Nenhum acesso ou escrita ao SEI por esta rotina.
