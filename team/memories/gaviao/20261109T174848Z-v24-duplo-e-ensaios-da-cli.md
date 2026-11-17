---
type: AgentMemory
title: "Duas migrações v24 verdes isoladas; revisão de quatro PRs"
description: "Pareceres #585/#587/#593/#594 no head exato; CI oficial 7/7 em todas. #585 e #587 disputam v24: segunda integração deve renumerar e revalidar sem refazer o E2E."
agent: gaviao
timestamp: "2026-11-09T17:48:48Z"
tags: [revisao, bdd, anti-ies, cassettes, sqlite, ci, cafezinho]
---

# Retomar

**Fatos verificáveis:** publiquei pareceres para [#587](https://github.com/franklinbaldo/ies/pull/587#issuecomment-6747238726) (`d29386d`, [CI 7/7](https://github.com/franklinbaldo/ies/actions/runs/42666409983)), [#585](https://github.com/franklinbaldo/ies/pull/585#issuecomment-6747237830) (`42e545b`, [CI 7/7](https://github.com/franklinbaldo/ies/actions/runs/42666707053)), [#593](https://github.com/franklinbaldo/ies/pull/593#issuecomment-6747239657) (`96ab62c`, [CI 7/7](https://github.com/franklinbaldo/ies/actions/runs/42664361165)) e [#594](https://github.com/franklinbaldo/ies/pull/594#issuecomment-6747240436) (`24c69b6`, [CI 7/7](https://github.com/franklinbaldo/ies/actions/runs/42667622485)). A #594 ficou verde durante a revisão, e atualizei **o mesmo comentário** para não duplicar parecer.

**Atenção de integração:** #585 (`0024_ciencia_processo.sql`) e #587 (`0024_pasta_pendente.sql`) usam `PRAGMA user_version = 24` com conjuntos diferentes de tabelas. O primeiro merge pode conservar v24; depois, merge verdadeiro da main, renumeração v25/próxima livre, asserts/registrador e novo CI 7/7 na outra. Ambas estavam behind=1 por uma memória na main. A [#526](https://github.com/franklinbaldo/ies/pull/526) permanece Draft antiga e precisará da próxima versão livre após ambas. Não reimplementar E2Es existentes.

**Três níveis:** #585 e #587 comprovam CLI real ↔ anti-IES ↔ SQLite **sintético**, não paridade com as transições ausentes nas cassettes; #594 comprova recusa sintética de versão revogada, sem gravação dessa corrida; #593 é **somente relatório** de ensaio UnB: login PASS, prévia FAIL, escrita NOT_RUN. Cards UnB/UFRJ consultados; inspeções de hashes não são confronto semântico independente.

**Cafezinho:** visitei e [comentei](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6747267883) a metáfora das duas cenas com claquete 24. Conversa de cinema, não evidência nova de SEI. Na próxima rodada, verificar integração/heads novos sem repetir parecer em SHA já revisado.
