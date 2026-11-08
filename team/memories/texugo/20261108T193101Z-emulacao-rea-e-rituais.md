---
type: AgentMemory
title: "Emulação seletiva e REA versionada: primeiras integrações"
description: "Primeira propriedade sintética compartilhada do anti-IES, skill REA incorporada e rituais explicitamente preservados no AGENTS.md."
agent: texugo
timestamp: "2026-11-08T19:31:01Z"
tags: [anti-ies, rea, emulacao, rust, antichurn, rituais]
---

# Retomada

**Fato:** PR [#499](https://github.com/franklinbaldo/ies/pull/499) integrada à main com CI oficial 7/7: `infra_hash` sintético unificado entre barra do processo e calendário, preservando navegação, sem autenticação real nem alteração SQLite. Skill REA v61 em `.agents/skills/reverse-engineer-anything/`, com instruções e referências oficiais upstream versionadas; o servidor MCP REA e engines **não foram instalados nem executados**, exigem configuração do host se necessários.

**Fato:** PRs [#494](https://github.com/franklinbaldo/ies/pull/494) e [#498](https://github.com/franklinbaldo/ies/pull/498) integradas com CI oficial 7/7: AGENTS.md determina que rituais do proprietário (Cafezinho, memórias, BDD, cassettes, gates) não são churn; documenta a separação entre estado funcional e propriedades emuladas. O arquivo ficou dentro de 20.000 bytes (19.999 antes do merge).

**Continuar:** issues [#496](https://github.com/franklinbaldo/ies/issues/496) e [#497](https://github.com/franklinbaldo/ies/issues/497) abertas; aplicar a separação incrementalmente usando BDD da CLI já predefinido, procurar cassettes primeiro, sem REA/live como gate. Não introduzir modelo genérico SeiProps nem subcódigos. Teste sintético ≠ confronto independente com cassettes ≠ live.

**Cafezinho:** na [issue #388](https://github.com/franklinbaldo/ies/issues/388), os colegas enfatizaram que registro vazio observado difere de registro ausente. Isso reforça a disciplina de distinguir fato, hipótese e desconhecido, não constitui evidência externa do SEI.
