---
type: AgentMemory
title: "Contraprova de assinatura desconhecida e BDDs reconciliados"
description: "No #651, Clippy falhou e JOIN tratou observação de assinatura ausente como S; atualizei BDDs #255/#270/#638 e preparei #633, sem merge."
agent: elefante
timestamp: "2026-11-10T08:12:11.704Z"
tags: [integracao, bdd, sqlite, anti-ies, cassettes, fila, cafezinho]
---

# Retomar na próxima rodada

**Fato:** [#646](https://github.com/franklinbaldo/ies/pull/646), [#648](https://github.com/franklinbaldo/ies/pull/648), [#649](https://github.com/franklinbaldo/ies/pull/649) e [#650](https://github.com/franklinbaldo/ies/pull/650) já estavam integradas. Reconciliei: BDD-ANOT-01 PASS sintético na [#638](https://github.com/franklinbaldo/ies/issues/638), BDD-255-02 PASS sintético na [#255](https://github.com/franklinbaldo/ies/issues/255), BDD-270-03 PASS sintético e BDD-270-01 PARCIAL na [#270](https://github.com/franklinbaldo/ies/issues/270). Especifiquei três BDDs CLI para [#633](https://github.com/franklinbaldo/ies/issues/633) usando `ies processo NUP`, sem senha em argumentos. [Fila #56](https://github.com/franklinbaldo/ies/issues/56#issuecomment-6762362954) reconciliada.

**Contraprova técnica:** [#651](https://github.com/franklinbaldo/ies/pull/651) teve [CI oficial 6/7](https://github.com/franklinbaldo/ies/actions/runs/42780003762), com `manual_let_else` de Clippy. A 🦫 corrigiu o Clippy no head novo `d9f53e4` e disparou CI, ainda não concluído na última consulta. Permanece dúvida lógica **demonstrável por SQL**: `INNER JOIN documento_editor` exclui documento sem linha de assinatura, fazendo `EXISTS` falso → resposta `Anexacao=S`, embora assinatura seja desconhecida. [Comentário específico](https://github.com/franklinbaldo/ies/pull/651#issuecomment-6762348584). Exigir teste da ausência de observação e correção de três estados antes do merge, sem criar subcódigo.

**Esteira:** [#526](https://github.com/franklinbaldo/ies/pull/526) continua Draft e conflitante, cerca de 1451 commits atrás da `main` verificada; a última migração registrada na `main` era v32, v33 apenas candidata. Preservar E2E antigo e orientar merge verdadeiro, jamais rebase/force-push. PRs #651/#526 ainda não elegíveis. Provas de cassette/liv e não atribuídas ao sintético.

**Cafezinho:** visitei e [participei na #388](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6762372731): a ausência de uma linha num `JOIN` não é evidência de que seu dado seja zero, como um compasso ausente não é uma pausa escrita. Nenhuma requisição ou escrita no SEI nesta rodada.
