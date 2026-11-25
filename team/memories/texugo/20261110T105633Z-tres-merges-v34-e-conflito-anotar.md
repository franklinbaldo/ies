---
type: AgentMemory
title: "Três integrações sintéticas e conflito localizado da anotação"
description: "Integrei #656 (SQLite v34), #660 e #659 com CI 7/7 e rápido verdes em cada head; #654 exige reconciliar Comentar e Anotar, sem refazer testes."
agent: texugo
timestamp: "2026-11-10T10:56:33Z"
tags: [antichurn, integracao, sqlite, bdd, cassettes, ci]
---

# Próxima rodada

**Fatos verificados:** [#656](https://github.com/franklinbaldo/ies/pull/656) entrou por `68bc003`, head `8da29f0`, CI oficial [7/7](https://github.com/franklinbaldo/ies/actions/runs/42801720432) e rápido verde; criou o catálogo SQLite **v34** e a atribuição causal de marcador com CLI real ↔ anti-IES ↔ SQLite. A [#660](https://github.com/franklinbaldo/ies/pull/660) entrou por `a760254`, head `33ffa24`, CI [7/7](https://github.com/franklinbaldo/ies/actions/runs/42802612303) e rápido verde; sua novidade é regressão **HTTP/SQLite** de protocolo repetido, não E2E novo da CLI. A [#659](https://github.com/franklinbaldo/ies/pull/659) entrou por `30e9e0e`, head `9e46c44`, CI [7/7](https://github.com/franklinbaldo/ies/actions/runs/42799742274) e rápido verde: prévia de comentários e segunda invocação da CLI sem POST. Nenhuma validação live/SEI foi executada e integridade de cassette não equivale a confronto bilateral.

**Bloqueio material:** [#654](https://github.com/franklinbaldo/ies/pull/654) tem head `a4c494b` com CI oficial 7/7 e rápido verdes, mas GitHub retornou `mergeable_state=dirty` depois da #659. O conflito reproduzível é `src/cli_processo.rs`: conservar `Comentar` novo da main e adicionar `Anotar --confirmar` da #654 no merge verdadeiro, depois CI do SHA combinado. [Registrei o despacho](https://github.com/franklinbaldo/ies/pull/654#issuecomment-6765517905). A #526 ainda é Draft e conflita com versões históricas; v35 é apenas próxima candidata após a v34, não reserva.

**Cafezinho:** a [🦊 notou](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6765261997) que a mesma rota alterna 302/200 após mudança de estado. Importante testar antes/depois em vez de congelar formato; isso é uma lição de modelagem, não uma prova live.

**Antichurn:** não exigir REA/live para integrar E2E sintético verde; não ignorar conflito real ou novo head sem CI. Não houve CI pós-merge nesta rodada.
