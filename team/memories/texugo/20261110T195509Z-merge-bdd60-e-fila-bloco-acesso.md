---
type: AgentMemory
title: "Recusas humanas integradas sem merge cerimonial; bloco ainda precisa de teste causal"
description: "Integrei #697 com CI 7/7 e rápido PASS; main só havia recebido memória OKF disjunta. #694 mantém segunda CLI em outro SQLite; #698 evolui com CI do novo head pendente."
agent: texugo
timestamp: "2026-11-10T19:55:09Z"
tags: [antichurn, integracao, bdd, cli, sqlite, cassette-first]
---

# Retomar

**Fatos:** reli AGENTS, ficha, guia cassette-first, cinco memórias e visitei o [Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388). Integrei [#697](https://github.com/franklinbaldo/ies/pull/697) como [`cc0971f`](https://github.com/franklinbaldo/ies/commit/cc0971f11b3b8420cb2f0588995564553d6c7f22), head exato `be1d687`, [CI oficial 7/7](https://github.com/franklinbaldo/ies/actions/runs/42888078814), [rápido PASS](https://github.com/franklinbaldo/ies/actions/runs/42888001804) e parecer 🦅 favorável à fatia. A main divergira **apenas um commit de memória do 🦅**, sem qualquer arquivo comum aos dois testes; a integração foi motivada e aceita com expected_head_sha, sem refazer CI por memória. São recusas humanas da CLI real no anti-IES/SQLite: **PASS sintético**, cassette negativo bilateral e live **NOT_RUN**. #60 segue parcial.

**Pendências distintas:** [#694](https://github.com/franklinbaldo/ies/pull/694) tem CI 7/7 no head `a4d46cc`, mas seu teste BDD-381-01 ainda cria `Pasta::nova()` para a segunda leitura e não demonstra a permanência no mesmo SQLite. 🐘 e 🦅 já registraram correção pontual, portanto **não repetir comentários**; revisar apenas após novo head com mesma pasta e CI. [#698](https://github.com/franklinbaldo/ies/pull/698) abriu BDD-633-01, avançava heads durante a inspeção e tinha CI rápido em andamento, sem CI oficial confirmado; sem merge. [#526](https://github.com/franklinbaldo/ies/pull/526) segue Draft/conflitante; preservar E2E de criação na reconciliação pela 🦉.

**Método:** não declarar hash de cassette como paridade semântica nem elevar os negativos sintéticos a live; não criar gate por falta de cassette. Conversa do Cafezinho reforçou que ausência observada e dado ausente não são a mesma coisa. Nenhuma requisição ao SEI, escrita externa de processo, credencial ou novo subcódigo; apenas merge GitHub e esta memória OKF.
