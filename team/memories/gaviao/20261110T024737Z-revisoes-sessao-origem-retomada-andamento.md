---
type: AgentMemory
title: "Revisões da sessão revogada, recusa tipada, retomada e andamento"
description: "Quatro heads revisados; #630 7/7 sem conflito antes da memória; #627/#624/#622 dependem de atualizar main e distinguir provas sintéticas."
agent: gaviao
timestamp: "2026-11-10T02:47:37Z"
tags: [revisao, bdd, anti-ies, cassette, sqlite, ci, cafezinho]
---

# Para retomar

**Fatos da revisão:** [#630](https://github.com/franklinbaldo/ies/pull/630#issuecomment-6757520013), head `e0bcd55`: BDD-60-05 executa CLI real contra anti-IES e SQLite; recusa HTTP 403 quando a sessão é revogada entre árvore e lista, sem GET do corpo e sem cancelar documentos. [CI #42744514511](https://github.com/franklinbaldo/ies/actions/runs/42744514511) 7/7 e rápido verde; `behind=0` **antes desta memória**, reconferir após commit.

[#627](https://github.com/franklinbaldo/ies/pull/627#issuecomment-6757520434), head `6f2eabf`: recusa de GET de retomada com 302/login não comprova autoria do estado posterior; testes internos PASS, mas E2E CLI↔anti-IES↔SQLite BDD-269-03 PARCIAL; CI oficial 7/7, `behind=26` na conferência.

[#624](https://github.com/franklinbaldo/ies/pull/624#issuecomment-6757520887), head `e7f710c`: causa de origem serializada diretamente por Serde, CLI pré-rede PASS e CI 7/7; `behind=10`. O BDD-620-01 da issue contradiz autorização posterior de andamento; 🐘 precisa reconciliar. Preservar regra nova das sete instalações para futuras liberações e exceções das quatro anteriores.

[#622](https://github.com/franklinbaldo/ies/pull/622#issuecomment-6757521385), head `857e7fb`: um POST sintético, 503 pós-escrita, saída 3 e segunda CLI de leitura; CI 7/7 mas `behind=26`. Todos precisam de merge verdadeiro e novo 7/7 no head reconciliado para integrar.

**Classificação:** PASS sintético onde executado, inspeção de card ≠ confronto semântico independente, e nenhuma nova validação live nesta rodada. Não usar ausência da corrida na cassette como veto. [#526](https://github.com/franklinbaldo/ies/pull/526) segue Draft e conflitante, head antigo.

**Cafezinho:** após os pareceres, li [#388](https://github.com/franklinbaldo/ies/issues/388) e [comentei](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6757532709) a diferença entre destruir o filme e perder autorização para assisti-lo: analogia de estado documental versus sessão, não evidência do SEI. Sem implementação, merge, produção ou escrita no SEI.
