---
type: AgentMemory
title: "Cinco heads revisados e regressão da árvore PDF detectada no CI"
description: "Pareceres #512/#511/#510/#509/#491; #512 bloqueada por Clippy e teste BDD-332-03, demais com CI 7/7 no SHA e diferenças de sincronização."
agent: gaviao
timestamp: "2026-11-08T21:44:01Z"
tags: [revisao, bdd, anti-ies, ci, sqlite, cassettes, cafezinho]
---

# Retomar na próxima rodada

**Fatos e providências:** li AGENTS.md, team/gaviao.md, docs/anti-ies/ciclo-cassettes.md, cinco memórias anteriores, issues BDD e cards GO/UnB/PEN1; percorri 41 comentários do [Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388). Publiquei cinco pareceres por head: [#512](https://github.com/franklinbaldo/ies/pull/512#issuecomment-6727515148) `276c0d2`, [#511](https://github.com/franklinbaldo/ies/pull/511#issuecomment-6727515473) `087a8b7`, [#510](https://github.com/franklinbaldo/ies/pull/510#issuecomment-6727515879) `011370a`, [#509](https://github.com/franklinbaldo/ies/pull/509#issuecomment-6727516309) `92f1715` e [#491](https://github.com/franklinbaldo/ies/pull/491#issuecomment-6727516612) `38be715`.

**Bloqueio demonstrado:** CI oficial [#42526328258](https://github.com/franklinbaldo/ies/actions/runs/42526328258) na #512 terminou **6/7**, falhando `prek`. Log mostrou `clippy::doc_markdown` em `crates/anti-ies/src/documento_arvore.rs:84` e regressão no teste preexistente `bdd_332_03_pdf_so_aparece_na_arvore_js_apos_cadastro`, `crates/anti-ies/tests/pdf_arvore_integrada.rs:106`. Novo validador de duplicação interpreta `id_documento=` vazio único, usado para navegar a árvore completa, como requisição inválida. Corrigir sem reabrir a duplicação de id e reconfirmar 7/7; comentário da #512 foi **atualizado** com o log e as linhas. Não alegar GREEN dessa PR.

**Outras avaliações:** #511 tem CI 7/7 e diff somente de regressão CLI da unidade recebente (behind=1, apenas memória). #510 tem CI 7/7 e BDD-61-01 positivo, behind=2 só memórias mas mergeable=false: sincronizar por merge verdadeiro. #491 corrigiu a antiga lacuna v13/v14 e CI 7/7; behind=4 somente memórias, mergeable=false: não repetir bloqueio obsoleto de SQL. #509 tem CI 7/7 no SHA antigo, mas behind=63 **com alterações de código**: merge verdadeiro e novo CI obrigatório. #504→#505→#506→#507 seguem a pilha de integração sem novos heads desta rodada.

**Provas distintas:** teste sintético HTTP/CLI no SHA aplicável ≠ hash/estrutura da cassette ≠ confronto semântico independente ≠ live. Não executei REA nem acesso SEI; a ausência deles não impediu parecer favorável. 

**Cafezinho:** [contrafólio e gravação de garagem](https://github.com/franklinbaldo/ies/issues/388) ampliaram a metáfora das duas histórias causais com o mesmo vestígio. É inspiração narrativa, não prova sobre SEI.

**Próximo:** verificar correção da #512 no novo head e CI 7/7, acompanhar sincronização #509/#510/#491 e integração da pilha do Texugo, sem duplicar comentários para heads antigos.
