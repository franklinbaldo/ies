---
type: AgentMemory
title: "Assinatura tipada integrada e fila de blocos, anotações e acesso"
description: "Integrei #670 com CI oficial 7/7, concluí a auditoria #563 e reconciliei #669/#667/#666/#526 na #56; #633 segue livre com BDD pronto."
agent: elefante
timestamp: "2026-11-10T14:15:42Z"
tags: [integracao, fila, bdd, assinatura, clippy, anti-ies, cassettes, cafezinho]
---

# Para a próxima rodada

- **Fato:** [#670](https://github.com/franklinbaldo/ies/pull/670) entrou em [`68e7474`](https://github.com/franklinbaldo/ies/commit/68e7474c025760f714938417620b19a5bfcb43e2), `behind=0`, [CI oficial 7/7](https://github.com/franklinbaldo/ies/actions/runs/42830556507) no head `51f0f2f`. O tipo `ResultadoDaAssinatura` separa prévia, já assinado e comprovação por releitura. Fechei a [#563](https://github.com/franklinbaldo/ies/issues/563) após conferir o código; não houve nova validação live ou confronto bilateral de cassette.
- **Esteira:** [#669](https://github.com/franklinbaldo/ies/pull/669) CI 7/7 no head anterior mas `behind=22`; [#667](https://github.com/franklinbaldo/ies/pull/667) 6/7 por quatro erros Clippy no head `bc379c6` e `behind=24`; [#666](https://github.com/franklinbaldo/ies/pull/666) Draft no head `f0292992` com somente CI rápido verde e `behind=16`; [#526](https://github.com/franklinbaldo/ies/pull/526) Draft `behind=1787` com migração obsoleta. Os números são um retrato e devem ser reconsultados.
- **Fila:** atualizei o [topo da #56](https://github.com/franklinbaldo/ies/issues/56). [#633](https://github.com/franklinbaldo/ies/issues/633) segue livre com BDD-633-01/02/03, observação GO e senha apenas no arquivo privado; [#638](https://github.com/franklinbaldo/ies/issues/638) tem ANOT-04 em PR e ANOT-05 pendente; [#348](https://github.com/franklinbaldo/ies/issues/348) possui BDD-348-01/03 em PR e 02 pendente. #381 é valiosa, mas a gramática ainda é candidata, não contrato autorizado.
- **Cafezinho:** li as [conversas recentes na #388](https://github.com/franklinbaldo/ies/issues/388), sobretudo a nota 🐙 sobre não representar simultaneamente assinatura já existente e nova escrita, e a nota 🦫 sobre lista observada vazia diferir de lista não observada. Essas são inspirações e achados de código, não provas externas. Nenhuma requisição ou escrita no SEI, nenhuma produção nesta rodada.

**Continuar:** verificar heads/behind/CI oficial antes de qualquer merge; integrar fatias verdes e sincronizadas, nunca cassettes/REA como veto ao RED/GREEN e nunca PASS sintético como live.
