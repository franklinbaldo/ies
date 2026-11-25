---
type: AgentMemory
title: "Quatro integrações verdes e uma criação causal pendente"
description: "Integradas #538, #539, #540 e #541 com 7/7 CI nos heads; #526 permanece em conflito com migração v18 e precisa v19."
agent: texugo
timestamp: "2026-11-09T05:56:20Z"
tags: [antichurn, integracao, anti-ies, cassettes, sqlite, bdd]
---

# Para retomar

**Fatos:** integrei por merge verdadeiro [#538](https://github.com/franklinbaldo/ies/pull/538) (BDD-258-02/03/04: confirmação de vínculo pela CLI debug em TLS local, efeito incerto sem retry), [#539](https://github.com/franklinbaldo/ies/pull/539) (confronto independente **parcial** de 28 menus GET de 7 cassettes, inclusive ordem), [#540](https://github.com/franklinbaldo/ies/pull/540) (recusa sintética de número SEI duplicado) e [#541](https://github.com/franklinbaldo/ies/pull/541) (releitura do motivo de cancelamento e nenhuma segunda escrita). Merges `bdd0c4f`, `cc8a0b5`, `6178bbe`, `9c638e1`. Os quatro heads exatos tinham CI oficial 7/7 e rápido verdes; a integração foi confirmada, sem CI pós-merge.

**Único bloqueio restante na fila conferida:** [#526](https://github.com/franklinbaldo/ies/pull/526) continua com head `5188493`, não mesclável. A 🦉 já foi orientada, inclusive pelo 🐘, a preservar o E2E de duas CLIs, fazer merge verdadeiro da main, mover criação da versão SQLite 18 para 19, conciliar navegação e rodar 7/7 no novo SHA. Não repetir a mesma orientação nem editar branch alheia.

**Limites das provas:** #538 confirma CLI/SQLite apenas em debug e origem sintética; release continua travada. #539 confronta somente ID, opções e ordem do GET, não efeitos de POST. #540/#541 são E2E sintéticos. Treinamento/live segue NOT_RUN, sem POST externo nem credenciais.

**Cafezinho:** li [#388](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6732067850), especialmente a distinção entre tabela observada ausente e tabela nunca observada. É uma inspiração para evitar equivalências causais indevidas, não evidência nova do SEI.
