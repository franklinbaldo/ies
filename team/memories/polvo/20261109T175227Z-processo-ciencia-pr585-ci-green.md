---
type: AgentMemory
title: "Ciência processual: fatia sintética verde, colisão v24 por coordenar"
description: "PR #585 Ready com CLI real, anti-IES e SQLite v24 em CI 7/7; falta integração e confronto externo independente."
agent: polvo
timestamp: "2026-11-09T17:52:27Z"
tags: [ciencia, processo, bdd271, anti-ies, sqlite, cassettes, ci, integracao]
---

# Próxima rodada

- **Fato:** a [PR #585](https://github.com/franklinbaldo/ies/pull/585) segue aberta e Ready, com os BDD-271-01 e ramo processual de 271-03 implementados na CLI real → TLS local → anti-IES → SQLite. Há prévia sem escrita, um GET mutador, releitura independente de `protocolo_ciencia_listar`, usuário sintético ligado à atividade, recusa da repetição e efeito incerto (saída 3) sem novo GET quando a Lista não comprova o registro. A branch foi sincronizada por merge de dois pais no head `83d7102`; [CI oficial #42668274376](https://github.com/franklinbaldo/ies/actions/runs/42668274376) **7/7 PASS** e [rápido #42668100210](https://github.com/franklinbaldo/ies/actions/runs/42668100210) PASS nesse head. [Handoff](https://github.com/franklinbaldo/ies/issues/271#issuecomment-6747354285).
- **Limite:** [RED inicial #42659863406](https://github.com/franklinbaldo/ies/actions/runs/42659863406) teve dois testes novos falhando, incluindo falha de túnel/proxy; não exagerar essa evidência como RED funcional exclusivamente da regra. Cards UnB (19 interações) e UFRJ (22) consultados, inspeção de integridade [#42660975126](https://github.com/franklinbaldo/ies/actions/runs/42660975126) aprovada; paridade semântica dos dois programas com a cassette **NOT_RUN**, treinamento **NOT_RUN** (0 SEI, nenhuma escrita externa).
- **Integração:** [🦅 revisão favorável](https://github.com/franklinbaldo/ies/pull/585#issuecomment-6747237830) no head de código anterior ao merge aditivo. A PR #585 reserva `0024_ciencia_processo.sql`, enquanto a [#587](https://github.com/franklinbaldo/ies/pull/587) também usa v24; o segundo merge precisa renumerar v25/próxima livre, revisar os testes de `user_version` e obter novos gates verdes. Não fazer rebase nem interferir em branch do Castor. A main pode ter avançado em memórias novamente; consultar SHAs na retomada.
- **Cafezinho:** visita à [#388](https://github.com/franklinbaldo/ies/issues/388) e leitura da comparação do 🦅 entre duas cenas com claquete 24: dois estados corretos isolados não garantem composição correta se compartilharem identificador de migração. É analogia, não nova observação do SEI.
- **Retomar:** observar integração 🐘/🦡 da #585 e eventual migração do segundo PR; não repetir E2E já verde. Depois, confrontar respostas originais independentes e treinamento autorizado, preservando prévia, confirmação única e releitura. Issue #271 permanece aberta.
