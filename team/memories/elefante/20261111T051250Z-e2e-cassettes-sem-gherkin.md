---
type: AgentMemory
title: "Fim do BDD/Gherkin: E2E guiado por cassettes é o método"
description: "Atualizados os prompts das oito automações, roadmap #56 e fila #716; PR #758 remove Cucumber/Gherkin e atualiza skills, AGENTS e rotinas, ainda aguardando CI e merge."
agent: elefante
timestamp: "2026-11-11T05:12:50Z"
tags: [e2e, cassettes, gherkin, automacoes, ablação, antipatterns]
---

# Ponto de retomada

- **Decisão expressa do proprietário:** acabar com BDD/Gherkin/Cucumber. Não escrever nem exigir cenários Dado/Quando/Então. Prioridade da fila continua sendo eliminar antipatterns (#716), com teste E2E via cassettes originais para operações Web.
- **Material executado:** prompts das oito rotinas ChatGPT atualizados sem alteração de horários/estado; [roadmap #56](https://github.com/franklinbaldo/ies/issues/56) atualizado; orientação registrada na [#716](https://github.com/franklinbaldo/ies/issues/716#issuecomment-6787656860).
- **PR [#758](https://github.com/franklinbaldo/ies/pull/758):** remove `tests/bdd_cli.rs`, arquivo `.feature`, Cucumber/futures e 87 pacotes órfãos do lock; atualiza AGENTS, oito fichas, docs, scripts e skill nova `.agents/skills/cassette-e2e/SKILL.md`. Alterações na branch, **não afirmar merge ainda**. TOML e shell passaram por validação leve; Rust CI 7/7 e sincronização definitiva ainda faltam.
- **Não apagar regressões Rust úteis em massa:** nomes históricos `bdd_` devem deixar de ser fonte normativa; migrar para E2E cassette-backed quando houver cobertura equivalente. Um teste sintético não prova confronto bilateral com a fonte.
- **Cafezinho:** #388 visitado nesta rodada, com leituras recentes sobre conflitos de tela e resposta observada. Nenhuma execução live/REA realizada pelo Elefante.
- **Próxima rodada:** conferir base da #758, sincronizar sem rebase/force-push, CI oficial do SHA exato e integração; respeitar alterações paralelas de outros agentes.
