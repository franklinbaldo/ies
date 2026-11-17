---
type: AgentMemory
title: "Histórico cancelado integrado, REDs diagnosticados e v19 concorrente"
description: "Integrei #548 com 7/7 CI no head; #546 tem RED de ciência e migração v19 concorrente com #526, #547 falhou em Rustfmt."
agent: texugo
timestamp: "2026-11-09T08:00:22Z"
tags: [antichurn, integracao, anti-ies, sqlite, cassettes, ci]
---

# Retomar

**Fatos:** integrei por merge verdadeiro [#548](https://github.com/franklinbaldo/ies/pull/548), commit `26043fdb`, head `8dde0a8`: histórico de versões antes publicado deixa de oferecer links após cancelamento persistido no mesmo SQLite; outra identidade permanece acessível. CI oficial [#42582031611](https://github.com/franklinbaldo/ies/actions/runs/42582031611) teve 7/7 jobs verdes, rápido verde e branch estava 0 commits atrás. A prova **nova** é HTTP/SQLite sintética, não novo E2E CLI; cards GO/UnB tiveram integridade inspecionada, mas não contêm esse estado pós-cancelamento. Cassette semântico e treinamento: NOT_RUN.

**Desbloqueios sem duplicação:** [#547](https://github.com/franklinbaldo/ies/pull/547) head `7d45a67`, CI rápido [#42580801547](https://github.com/franklinbaldo/ies/actions/runs/42580801547) falhou só em Rustfmt de `tests/aies_atribuicao_cli.rs:235`; [orientei correção mínima](https://github.com/franklinbaldo/ies/pull/547#issuecomment-6735101007) antes de CI/RED real. [#546](https://github.com/franklinbaldo/ies/pull/546) head `87973b9`, CI [#42580178645](https://github.com/franklinbaldo/ies/actions/runs/42580178645) 6/7, teste novo da ciência falha por proxy sem requisições; [registrei diagnóstico](https://github.com/franklinbaldo/ies/pull/546#issuecomment-6735126244). A #546 usa `0019_ciencia_documental.sql`; a velha [#526](https://github.com/franklinbaldo/ies/pull/526), draft e não mesclável no `5188493`, pretende usar v19 para processo criar. Integrar primeiro o que ficar verde e mover a outra migração para a próxima versão livre, sem atrasar trabalho funcional. Não editar branches alheias nem repetir instruções antigas da #526.

**Cafezinho:** li a [#388](https://github.com/franklinbaldo/ies/issues/388), inclusive o mensageiro e a partitura `tacet`: autorização/endereçamento não certifica que um efeito ocorreu. Analogia narrativa, não prova do SEI. Sem escrita em produção, credenciais, novos subcódigos ou validação live.
