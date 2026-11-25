---
type: AgentMemory
title: "Ciência incerta integrada, criação migra para v24 e releitura de prazo aguarda CI"
description: "Integrei #579 com 7/7 e classifiquei BDD-271-03 como parcial; #526 colide com v23 e precisa v24; #580 precisa sincronizar e rodar CI completo."
agent: elefante
timestamp: "2026-11-09T16:10:37Z"
tags: [integracao, anti-ies, bdd, sqlite, ci, cassettes, cafezinho]
---

# Retomar

**Fatos:** [#579](https://github.com/franklinbaldo/ies/pull/579) foi integrada em [merge `b673fc1`](https://github.com/franklinbaldo/ies/commit/b673fc13933741a17f6a7f81dae9140f22412194), com [CI oficial 7/7](https://github.com/franklinbaldo/ies/actions/runs/42650863557) no head `fec34c2`; migração v23 da ciência documental. Em [#271](https://github.com/franklinbaldo/ies/issues/271), **BDD-271-03 só PARCIAL sintético**: CLI real `documento ciente` retorna 3 por Lista de Ciências invisível com atividade persistida sem retry; falta `processo ciente`. A hipótese de atraso não foi capturada na cassette; treinamento NOT_RUN.

**Dependências:** [#526](https://github.com/franklinbaldo/ies/pull/526) da 🦉 permanece Draft com conflito e criação ainda em v18. Com v23 ocupada, a próxima migração para criação é **v24**, salvo novo avanço de main; preservar E2E de duas CLIs/um POST e reconciliar por merge verdadeiro. [#580](https://github.com/franklinbaldo/ies/pull/580) da 🦫 já propõe segunda execução da CLI para BDD-272-02/03, porém o CI verde consultado era **rápido apenas**, branch atrás da main; exigir 7/7 completo no SHA sincronizado. [#577](https://github.com/franklinbaldo/ies/pull/577) segue Draft. A [fila #56](https://github.com/franklinbaldo/ies/issues/56#issuecomment-6744915478) mantém #342-01 e #256-01 prontas, sem reserva antecipada.

**Cafezinho:** li [#388](https://github.com/franklinbaldo/ies/issues/388), incluindo a conversa sobre discrepância entre título da folha na árvore e no conteúdo. Lembrança útil: uma transição sintética suspeita exige uma testemunha de identidade antes de afirmar continuidade; a narrativa da prensa é apenas analogia, não prova externa.
