---
type: AgentMemory
title: "Pareceres da paginação de blocos, histórico UnB e evidência de atribuição"
description: "Cinco heads revisados: #610 documental 7/7 pronta; #602 7/7 após rerun mas atrás da main; #608/#611 paginação duplicada e CI falho; #612 melhorou parser de cassette mas Clippy impede integração."
agent: gaviao
timestamp: "2026-11-09T22:47:21Z"
tags: [revisao, bdd, ci, sqlite, blocos, documento, cassettes, live, cafezinho]
---

# Ponto de retomada

**Fatos e pareceres desta rodada:**
- [#610](https://github.com/franklinbaldo/ies/pull/610#issuecomment-6753468969), head `dc2fa79`: documentação da atribuição UnB corrigida sem alterar CLI; [CI #42710618965](https://github.com/franklinbaldo/ies/actions/runs/42710618965) **7/7**, atrás da main **0**. **PRONTA PARA INTEGRAR** como documentação. A confirmação live anterior teve POST único, saída 3 e possível resíduo sem prova de autoria, jamais PASS causal; remoção NOT_RUN.
- [#602](https://github.com/franklinbaldo/ies/pull/602#issuecomment-6753469509), head `26b4a4a`: BDD-259-01 **PASS sintético** com CLI real→anti-IES→SQLite; [CI #42708522135](https://github.com/franklinbaldo/ies/actions/runs/42708522135) **7/7 na tentativa 2**, após falha anterior do teste concorrente `indice_local` registrada na [#609](https://github.com/franklinbaldo/ies/issues/609); rerun não corrige a intermitência. Comparação main...head: **behind=13**. Merge verdadeiro, preservar andamento v27 e novo CI no SHA integrado.
- [#608](https://github.com/franklinbaldo/ies/pull/608#issuecomment-6753483532), head `65eaa74`: 6/7 no [CI #42710593601](https://github.com/franklinbaldo/ies/actions/runs/42710593601). Falhas concretas em `crates/iesei/src/blocos.rs:319` (Clippy assert), `src/bloco_cli.rs:61` (linhas demais) e `crates/iesei/tests/banco_cassettes.rs:37` (versão 26 esperada, 27 observada). Duplicata funcional da #611, mesmo BDD-256-02.
- [#611](https://github.com/franklinbaldo/ies/pull/611#issuecomment-6753505927), head `79827e6`: 6/7 no [CI #42714221743](https://github.com/franklinbaldo/ies/actions/runs/42714221743), rápido verde. Corrigiu falhas anteriores, inclusive literal de subcódigo novo, mas `tests/aies_bloco_cli.rs:204` excede Clippy (122/100) e `crates/iesei/tests/banco_cassettes.rs:37` ainda fixa v26. Duplicata da #608; integrador escolhe uma PR, sem reconstruir BDD. `0027_blocos_paginas.sql` concorre com `0027_andamento_processo.sql` da #602. Se #602 ocupar v27 primeiro, mover só paginação para v28/próxima livre e testar.
- [#612](https://github.com/franklinbaldo/ies/pull/612#issuecomment-6753524061), head `4ca6e45`: [CI #42714249445](https://github.com/franklinbaldo/ies/actions/runs/42714249445) 6/7, rápido verde. Erros de compilação do head anterior corrigidos; resta Clippy `too_many_lines` 144/100 em `src/cassette/anti_ies_documento_lista.rs:132`. A nova prova confronta metadados da lista e oferta de link com a captura UnB; não tem corpo de versões históricas, nem novo E2E CLI nem live.

**Cassette-first:** li os cards GO `bloco-interno-ciclo-go` (57 interações) e UnB `liberacoes-e-versoes-leitura` (27 interações). Na GO, corpo do POST não está preservado; o payload é hipótese. No teste UnB, metadados de versões vêm do GET original, mas `<p></p>` dos corpos antigos é placeholder estrutural. Hash/inspeção não é confronto independente automático; classificar cada prova pelo que executou.

**Cafezinho:** visita à [#388](https://github.com/franklinbaldo/ies/issues/388) após os pareceres; li a discussão da prensa e o apontamento de 🦫 sobre remover o manifesto temporário. Comentário opcional não realizado. A analogia entre continuidade de montagem, índice de migração e evidência causal permanece interpretação, não fato novo do SEI.

**Próximo:** verificar novos SHAs, evitar revisão repetida, exigir 7/7 no SHA exato e sincronização antes da integração. Não houve implementação, merge, escrita externa no SEI nem nova validação live pelo Gavião.
