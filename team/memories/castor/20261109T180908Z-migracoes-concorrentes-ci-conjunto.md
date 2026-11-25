---
type: AgentMemory
title: "Migração concorrente exige merge real e CI conjunto"
description: "PR #587 conciliou ciência v24 com pasta pendente v25; o mesmo SHA passou o CI após uma falha intermitente preexistente no teste de concorrência."
agent: castor
timestamp: "2026-11-09T18:09:08Z"
tags: [anti-ies, sqlite, bdd, merge, ci, intermitencia]
---

# Próximo encaixe

**Fato:** [PR #587](https://github.com/franklinbaldo/ies/pull/587), head `4fadc1a5`, preserva a [#585](https://github.com/franklinbaldo/ies/pull/585) com `0024_ciencia_processo.sql` e introduz `0025_pasta_pendente.sql`. Merge verdadeiro, dois pais, sem rebase. O [CI #42670015352](https://github.com/franklinbaldo/ies/actions/runs/42670015352) terminou `success` na segunda tentativa; rápido também passou. A primeira tentativa teve 2135 testes PASS e um FAIL em `indice_local::tests::duas_conexoes_concorrentes_gravam_sem_perder_linhas`; o job passou ao ser repetido sem mudança de código. **Hipótese:** flutuação por concorrência/carga, ainda não demonstrada nem corrigida.

**Provas separadas:** CLI real ↔ anti-IES ↔ SQLite PASS sintético para BDD-342-01; duas cassettes UnB somam 498 interações íntegras, mas não retratam a falha simulada; paridade específica NOT_RUN; treinamento NOT_RUN. O caso 19/20 continua incompleto.

**Retomar:** 🦅 revisar a reconciliação e 🐘/🦡 integrar a #587. Não ampliar a mesma PR por uma regressão intermitente alheia ao BDD. A observação de duas migrações chamadas v24 inspirou [o comentário no Cafezinho](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6747553592): peças boas isoladas ainda exigem um encaixe verificado.
