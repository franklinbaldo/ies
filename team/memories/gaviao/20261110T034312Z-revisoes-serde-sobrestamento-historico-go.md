---
type: AgentMemory
title: "Revisões da origem Serde, sobrestamento v30 e confronto GO"
description: "Novos heads #624/#627/#634 examinados: dois CI 7/7, um confronto GO ainda em CI; separar CLI sintética, cassette e live."
agent: gaviao
timestamp: "2026-11-10T03:43:12Z"
tags: [revisao, bdd, anti-ies, cassettes, sqlite, ci, cafezinho]
---

# Retomada

**#624**: head `6d65928`, [parecer](https://github.com/franklinbaldo/ies/pull/624#issuecomment-6758320219). BDD-620-01 agora usa atribuição restrita, corrigido pelo 🐘; Serde da causa tipada sem constante textual e CLI humano/JSON com zero conexão. CI oficial [#42750085523](https://github.com/franklinbaldo/ies/actions/runs/42750085523) **7/7** no head; `behind=0` antes desta memória. Revalidar sincronização após o commit OKF.

**#627**: head `f2ffca8`, [parecer](https://github.com/franklinbaldo/ies/pull/627#issuecomment-6758320780), CI [#42749469766](https://github.com/franklinbaldo/ies/actions/runs/42749469766) **7/7**. BDD-269-01/02/03 exercitados por CLI real, TLS loopback, anti-IES e SQLite v30, com uma segunda CLI observando estado. Corrida com 302/login demonstrada em teste interno, sem novo E2E da variante adversarial. `behind=2` na conferência: merge verdadeiro da main e novo 7/7 antes da integração. Cards UnB/GO disponíveis e inspecionados por hash no Actions; confronto semântico independente **NOT_RUN**.

**#634**: head `584b6d7`, [parecer](https://github.com/franklinbaldo/ies/pull/634#issuecomment-6758340705). Novo confronto independente com GET original de histórico GO, metadados/identidade preservados na projeção anti-IES/SQLite; corpos não capturados ficam placeholders não observados. [CI rápido #42750946625](https://github.com/franklinbaldo/ies/actions/runs/42750946625) verde; [CI oficial #42750946611](https://github.com/franklinbaldo/ies/actions/runs/42750946611) ainda **em execução** durante exame. Não atribuir PASS GO antes do job `prek` rodar o teste explicitamente ignorado. É teste diferencial, não novo E2E da CLI.

**#526** mantém head `5188493` Draft e conflitante, sem parecer duplicado; preservar E2E com duas CLIs/POST único e conciliar migração v18 histórica com a primeira versão livre (v31 caso #627/v30 integre antes).

**Provas e limites:** #624 e #627 têm PASS sintético delimitado, #634 confronto GO ainda não confirmado; nenhum live nesta rodada. Cafezinho [#388](https://github.com/franklinbaldo/ies/issues/388) visitado; leitura do comentário do 🐙 sobre autoria do efeito ([link](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6757840062)). A metáfora cinematográfica não substitui o teste causal. Sem implementação, merge, acesso SEI ou escrita de produção.
