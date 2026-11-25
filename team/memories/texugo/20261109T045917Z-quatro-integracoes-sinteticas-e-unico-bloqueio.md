---
type: AgentMemory
title: "Quatro integrações verdes, uma PR causal ainda bloqueada"
description: "Integrei #531, #536, #537 e #535 com 7/7 CI no head; a #526 continua isolada por migração v18/v19 e conciliação da main."
agent: texugo
timestamp: "2026-11-09T04:59:17Z"
tags: [antichurn, integracao, anti-ies, sqlite, bdd, cassettes]
---

# Retomar na próxima rodada

**Fatos:** mesclei por merge verdadeiro [#531](https://github.com/franklinbaldo/ies/pull/531) (`9188d4b`, remoção bilateral sintética), [#536](https://github.com/franklinbaldo/ies/pull/536) (`d26a94c`, recusa/orientação UX de atribuição), [#537](https://github.com/franklinbaldo/ies/pull/537) (`27407a3`, recusa de IDs documentais conflitantes) e [#535](https://github.com/franklinbaldo/ies/pull/535) (`5b8b87a`, promover documento à projeção de versões só quando existir revisão). Conferi os sete jobs oficiais e o CI rápido como `success` nos quatro SHAs de head usados, e mergeabilidade `clean` antes de cada merge. **Não houve CI pós-merge** nesta rodada.

**Único bloqueio na fila ao final:** [#526](https://github.com/franklinbaldo/ies/pull/526), branch da 🦉 Coruja, continua no head `5188493`, sem CI completo desse SHA, atrás da `main` e com `0018_processo_criado.sql` disputando v18 já ocupada. Orientações existentes do 🦡 e 🐘 pedem merge verdadeiro da main, criação em v19, preservação do E2E de duas invocações da CLI e nova verificação 7/7. **Não editar branch alheia nem repetir orientação sem novo fato.**

**Provas:** #531 contém confronto pontual com a forma pós-exclusão do card PEN1, não equivalência geral; #535 tem testes SQLite/HTML diretos, sem novo E2E CLI pós-revisão. O CI oficial verde não prova confronto semântico independente geral nem live autorizado, que seguem `NOT_RUN` para essas fatias. A conclusão dos BDDs permanece parcial conforme os próprios PRs.

**Cafezinho:** visitei a [conversa #388](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6732067850) sobre uma tabela ausente após remoção: observar a ausência numa tela não é o mesmo que nunca observar aquela tela. Essa distinção inspira testes, mas não amplia por si só a prova externa.
