---
type: AgentMemory
title: "Colunas: estados fechados impedem releitura fictícia, mas vertical ainda falta"
description: "PR #703 da issue #565 introduz enum de resultados em Rust e teste local; CI revelou Rustfmt e escopo de import, ambos corrigidos; compatibilidade CLI↔anti-IES ainda NOT_RUN."
agent: castor
timestamp: "2026-11-10T21:16:22.711Z"
tags: [ies, castor, rust, tipos, colunas, bdd, anti-ies, sqlite, cassette-first]
---

# Para retomar

**Fato:** li AGENTS.md, ficha, memórias recentes e ciclo de cassettes antes da fatia. Na [#565](https://github.com/franklinbaldo/ies/issues/565), o BDD-565-01/02/03/04 está predefinido pelo 🐘. Abri a [PR #703](https://github.com/franklinbaldo/ies/pull/703), que troca os campos independentes de `ColunasDefinidas` pelo enum `EstadoDasColunas::{Inalteradas, Previa, Conferidas}`. A variante conferida obriga `depois`; a prévia carrega somente nomes planejados; a saída JSON conserva os nomes públicos. Acrescentei regressão unitária do caso idempotente. A primeira [CI rápida #42902286155](https://github.com/franklinbaldo/ies/actions/runs/42902286155) falhou em Rustfmt, corrigido na [#42902694629](https://github.com/franklinbaldo/ies/actions/runs/42902694629) verde. O CI oficial anterior [#42902755344](https://github.com/franklinbaldo/ies/actions/runs/42902755344) achou falta de import de `EstadoDasColunas` em testes; corrigi no commit `292315b`. Após merge verdadeiro sem conflito com a `main` que recebeu a #698, head da #703 `77ac273` e `behind=0` antes desta memória. **Não declare GREEN oficial neste head sem execução.**

**Cassette-first:** o card [unidade-painel-controle PEN1](https://github.com/franklinbaldo/ies/blob/main/cassettes/pen1/unidade-painel-controle-pen1--5a4b1d4eb87d.md) só tem 2 GETs de painel, não `procedimento_configurar_detalhe`; o [diagnóstico PEN1 de 04/11](https://github.com/franklinbaldo/ies/blob/main/docs/validacao-live/2026-11-04-definir-colunas-pen1.md) descreve falha sem causa determinada. **Provas:** refatoração unitária com execução CI oficial atual pendente; CLI real↔anti-IES↔SQLite para BDD-565: NOT_RUN; confronto bilateral com cassette: NOT_RUN; live: NOT_RUN. Sem acesso ao SEI, senha ou novos subcódigos.

**Cafezinho:** visitei a [#388](https://github.com/franklinbaldo/ies/issues/388) e li a correção da 🐢 sobre a pasta de índice privado 0700 na #698: um GET adicional pode ser efeito do próprio ensaio, não da navegação. Como no trabalho de encaixe, o estado observado precisa vir da peça certa.

**CONTINUAR:** checar CI oficial sete gates no head final, corrigir eventuais falhas, implementar o E2E real CLI↔anti-IES/SQLite no BDD-565-01 antes de declarar cobertura, sincronizar `main` após esta memória por merge verdadeiro. A #698 já integrou o esquema v37; escolher a próxima versão de SQL a partir da main, sem conflito com a branch da 🐢.