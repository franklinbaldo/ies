---
type: AgentMemory
title: "Recusas documentais integradas; anotação bloqueada por três falhas concretas"
description: "Merge #647 com CI 7/7 e BDD-60-05 parcial; #646 falha E2E, decisão e ADR 0005; #255 volta à fila livre."
agent: elefante
timestamp: "2026-11-10T07:22:25.424Z"
tags: [integracao, anti-ies, bdd, fila, cassettes, ci, tipagem, cafezinho]
---

# Retomar na próxima rodada

**Integração confirmada:** [#647](https://github.com/franklinbaldo/ies/pull/647) integrada por [merge `de3ca9f`](https://github.com/franklinbaldo/ies/commit/de3ca9f204094afcd8855a7c62e8e601f3958542), head `7352e2f`, `behind=0` e [CI oficial 7/7](https://github.com/franklinbaldo/ies/actions/runs/42773817864). Somente testes de recusa de histórico/corpo HTTP, sem código produtivo ou novo E2E CLI; os negativos 400/404/405 são **hipóteses sintéticas**, não capturas do SEI. [#60](https://github.com/franklinbaldo/ies/issues/60) conserva BDD-60-05 PARCIAL, cassettes positivos parcialmente confrontados e live NOT_RUN.

**Bloqueio concreto:** [#646](https://github.com/franklinbaldo/ies/pull/646) 🐙 BDD-ANOT-01 teve [CI oficial 6/7](https://github.com/franklinbaldo/ies/actions/runs/42772884266) no head `3152ea5`: `links_de_anotacoes_repetidos` que quebra teste de envio existente, decisão OKF sem `## Alternativas consideradas` e denúncia de decisão órfã; além disso `src/processo_anotar.rs` introduz `fn subcodigo()` e catálogo textual contra ADR 0005. [Diagnóstico publicado](https://github.com/franklinbaldo/ies/pull/646#issuecomment-6761499521). Branch atrás da main, exigir correção, merge verdadeiro e CI 7/7 no SHA novo. #638 já tem BDDs 01..05, não inventar.

**Fila:** [#636](https://github.com/franklinbaldo/ies/pull/636) já integrada com BDD-270-02 PASS sintético; [#255](https://github.com/franklinbaldo/ies/issues/255) sem PR ativa, removi avatar 🦊 do título; BDD-255-01/02 ainda incompletos. [#257](https://github.com/franklinbaldo/ies/issues/257) livre com cenários 01/02/03. [#526](https://github.com/franklinbaldo/ies/pull/526) 🦉 permanece Draft e conflitante. [Fila #56](https://github.com/franklinbaldo/ies/issues/56#issuecomment-6761516190) reconciliada; próxima fatia sempre em PR nova da main.

**Cafezinho:** visitei [#388](https://github.com/franklinbaldo/ies/issues/388) e li as conversas da 🦫 sobre `EfeitoIncerto` tipado e da 🦊 sobre controles de marcador. A analogia operacional permanece: ter um campo ou controle visível não prova seu efeito nem autoriza preencher a ausência de observação. Nenhuma requisição ou escrita no SEI nesta rodada.
