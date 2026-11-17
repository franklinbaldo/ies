---
type: AgentMemory
title: "Merge de origem segura e orientação de não repetição na PR #615"
description: "PR #615 conciliada por merge verdadeiro com a guarda pré-rede da #617, preservando recusa em produção e orientação contra retry; CI do SHA conciliado em verificação."
agent: raposa
timestamp: "2026-11-10T00:16:06Z"
tags: [ux, processo-atribuir, merge, efeito-incerto, origem, anti-ies, ci]
---

# Retomar

**Fato:** a [PR #615](https://github.com/franklinbaldo/ies/pull/615) entrou em conflito após a #617. Reconciliei no head `1a9fb570` por merge verdadeiro de `main@ce81f293`, mantendo a guarda de origem de treinamento *antes de qualquer requisição* e a mensagem segura `não repita a confirmação; consulte o processo`. O diff final voltou a apenas três arquivos (CLI, teste E2E e contrato). O [CI rápido](https://github.com/franklinbaldo/ies/actions/runs/42726773558) passou; o [CI oficial](https://github.com/franklinbaldo/ies/actions/runs/42726818840) começou com sete jobs, ainda em execução na última observação. Conferir o novo head e 7/7 antes da integração.

**Separar evidências:** RED/GREEN sintético da PR #615 já comprovado no head anterior; nova composição aguarda CI; card UnB consultado, confronto causal com POST não reexecutado; treinamento não acessado. O resíduo de `aluno2` da execução anterior continua possível, não provado nem removido. Não repetir POST.

**Próxima prioridade, após integrar a #615:** [issue #620](https://github.com/franklinbaldo/ies/issues/620) já tem dois BDDs do 🐘 para remover o novo subcódigo manual introduzido pela #617 sem enfraquecer a guarda exata de origem. Manter trabalhos distintos.

**Cafezinho:** a [observação da 🐢 sobre o teste da cassette](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6754580305) reforça que os atributos de uma evidência também precisam de proveniência. Inspiração de método, não evidência nova da atribuição.
