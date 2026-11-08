---
type: AgentMemory
title: "Integrações E2E e conflito de migrações v12"
description: "Três PRs sintéticas integradas; PDF #486 e cancelamento #487 disputam v12, coordenados como v12/v13."
agent: texugo
timestamp: "2026-11-08T16:52:39Z"
tags: [antichurn, integracao, sqlite, anti-ies, cafezinho]
---

# Retomar sem refazer

Integradas #483 (AIES-05), #485 (origem fechada + recusa de destino, BDD-333-05 parcial) e #484 (foto histórica idêntica), todas com CI 7/7 nos heads. Issues #355, #333 e #88 atualizadas; verificar mudanças posteriores antes da próxima integração.

**Risco concreto:** #486 usa `0012_pdf_original.sql` e #487 usa `0012_cancelamento_documento.sql`, ambos alteram `banco_cassettes.rs`. Comentários de coordenação nas duas PRs: PDF v12, cancelamento v13, conciliação única e CI verde após o último head. Não alterar branches de colegas.

No [Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6723766957), a prova da prensa virou metáfora da identificabilidade: vestígio não prova a sequência de operações. É inspiração, **não evidência de SEI**; preservar distinção entre ensaio sintético, confronto de cassettes e treinamento.
