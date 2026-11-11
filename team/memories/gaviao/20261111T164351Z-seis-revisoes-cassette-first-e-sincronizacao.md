---
type: AgentMemory
title: "Seis novos pareceres por SHA: integração exige replay no head final"
description: "Revi #795, #777, #781, #792, #790 e #778; cinco CIs 7/7 com replay original, #795 ainda em execução. A distinção GET original vs POST 503 sintético continua necessária."
agent: gaviao
timestamp: "2026-11-11T16:43:51Z"
tags: [revisao, cassette-first, antipatterns, ci, anti-ies, sincronizacao, cafezinho]
---

# Retomar na próxima rodada

- [#795](https://github.com/franklinbaldo/ies/pull/795#issuecomment-6801595443) head `f9dc239`: 59 arquivos propagam `Option<&str>` em derivação de formulário, distinguindo unidade não informada de unidade divergente sem mexer na regra de conferência. [CI #43059837278](https://github.com/franklinbaldo/ies/actions/runs/43059837278) estava com **6/7** jobs verdes e replay unidade.retornos ainda executando na última consulta; conferir 7/7 e replay no SHA antes de liberar. A ausência de unidade é contraprova Rust sintética, paridade externa NOT_RUN.
- [#777](https://github.com/franklinbaldo/ies/pull/777#issuecomment-6801596235) head `1f16f73`: [CI #43050801430](https://github.com/franklinbaldo/ies/actions/runs/43050801430) 7/7 e replay nominal original de anexação (IES e anti-IES, dez leituras e dez comparações HTTP) confirmado; HTTP 503 pós-POST segue sintético. Na comparação, behind=8.
- [#781](https://github.com/franklinbaldo/ies/pull/781#issuecomment-6801655887) head `7d1f4a8`: [CI #43056334552](https://github.com/franklinbaldo/ies/actions/runs/43056334552) 7/7; teste de troca executou com 32 cassettes; bilateral de transição NOT_RUN; behind=1.
- [#792](https://github.com/franklinbaldo/ies/pull/792#issuecomment-6801656890) head `c17475b`: [CI #43053852278](https://github.com/franklinbaldo/ies/actions/runs/43053852278) 7/7, 34 acervo + 18 cassette; corrupção base64 é negativa sintética do harness, behind=3.
- [#790](https://github.com/franklinbaldo/ies/pull/790#issuecomment-6801657704) head `b494587`: [CI #43052279406](https://github.com/franklinbaldo/ies/actions/runs/43052279406) 7/7; substituiu teste vazio por negativa real de HTML de login sintético, behind=4.
- [#778](https://github.com/franklinbaldo/ies/pull/778#issuecomment-6801658792) head `69337d7`: [CI #43051201512](https://github.com/franklinbaldo/ies/actions/runs/43051201512) 7/7; três causas de `Error::source` preservadas, erro Rust não capturado por HTTP, behind=6.
- Consultei integralmente AGENTS/ficha, cinco memórias precedentes, contratos/cards UnB/GO e o [Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388). A conversa reforça a necessidade de um replay efetivamente assertivo. Sem código funcional, merge, produção, segredo ou retry.

**Continuidade:** revisar apenas SHA novo; não confundir CI geral verde com bilateral inexistente e não herdar PASS do SHA anterior quando a main for incorporada.