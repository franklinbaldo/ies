---
type: AgentMemory
title: "Troca de unidade: replay validado no SHA anterior, novo HEAD sob exame"
description: "Reconciliei a PR #781 com main@8d0432e2 sem conflito; CI rápido PASS no 0df361eb, oficial com replay executável ainda em andamento na consulta."
agent: raposa
timestamp: "2026-11-11T15:21:58Z"
tags: [ux, unidade-trocar, cassette-first, replay, ci, sincronizacao, cafezinho]
---

# Retomar na próxima rodada

**Fatos:** li [AGENTS.md](../../../AGENTS.md), ficha, cinco memórias e [Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388). O CI oficial [#43035581862](https://github.com/franklinbaldo/ies/actions/runs/43035581862) terminou **7/7 PASS no SHA anterior** `8ddd45ea`, incluindo `scripts/replay acervo` e teste nominal de troca com 32 cassettes e nenhuma recusa.

A `main` avançou nove commits, inclusive mudanças na apresentação de avisos da CLI. Construí merge verdadeiro `8ddd45ea` + `main@8d0432e2`, [novo HEAD `0df361eb`](https://github.com/franklinbaldo/ies/commit/0df361eb35658dc350a6412b57101d5bf5a81354), com diff limitado aos mesmos dois arquivos de teste. [CI rápido #43046854929](https://github.com/franklinbaldo/ies/actions/runs/43046854929) PASS; [CI oficial #43047004516](https://github.com/franklinbaldo/ies/actions/runs/43047004516) ainda rodava no último exame (6/7 jobs PASS, replay principal em andamento). Documentei a diferença de HEAD na [PR #781](https://github.com/franklinbaldo/ies/pull/781) e [issue #721](https://github.com/franklinbaldo/ies/issues/721#issuecomment-6799749824).

**Próximo passo:** conferir no **HEAD atual** resultado nominal de `unidade_trocar::acervo::replay_de_unidade_trocar_no_acervo`, 7/7 e revisão 🦅 antes de integrar por 🐘/🦡. IES↔ZIPs originais PENDENTE no novo SHA; anti-IES↔ZIPs e CLI↔anti-IES↔SQLite específicos da troca, além de live, NOT_RUN. Não herdar o PASS do SHA antigo nem refazer CI por alterações apenas memoriais.
