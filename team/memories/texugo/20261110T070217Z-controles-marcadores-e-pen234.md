---
type: AgentMemory
title: "Controles de marcadores e confrontos PEN2–PEN4 integrados"
description: "Integrei #644 e #645 com CI oficial 7/7 e rápido nos heads; #636 corrigiu tipos e documentação, mas novo CI ainda estava em andamento no momento da inspeção."
agent: texugo
timestamp: "2026-11-10T07:02:17Z"
tags: [antichurn, integracao, cassettes, bdd, ci, sqlite]
---

# Para retomar

**Fato:** [#644](https://github.com/franklinbaldo/ies/pull/644) integrou em `a63f1560` os controles de gerenciamento de marcadores na resposta GET sem habilitar POST. Head `b36bb467` teve CI oficial [#42768704998](https://github.com/franklinbaldo/ies/actions/runs/42768704998) 7/7 e CI rápido PASS. Revisão 🦅 favorável; alterações posteriores da main eram memórias sem sobreposição. Prova sintética de CLI real ↔ anti-IES ↔ SQLite, mas não comprova remoção/cadastro, nem confronto semântico com cassettes; live NOT_RUN.

**Fato:** [#645](https://github.com/franklinbaldo/ies/pull/645) integrou em `caa452f9`, head `2e0070b` com CI oficial [#42771931238](https://github.com/franklinbaldo/ies/actions/runs/42771931238) 7/7 e rápido PASS. Novos testes explícitos no CI fazem confronto **parcial e independente** das cassettes PEN2/PEN3/PEN4 para histórico, identidade e corpo da revisão observada por leitor IES + HTTP/SQLite anti-IES. Os corpos não capturados continuam desconhecidos, live NOT_RUN; nenhuma nova execução da CLI binária específica nesta fatia. As alterações da main após a base eram independentes (marcadores e memórias). Não houve CI pós-merge nas duas integrações.

**Fila:** [#636](https://github.com/franklinbaldo/ies/pull/636) no head `e267e12` corrigiu a divergência entre decisão/contrato e confirmação sintética, preserva causa tipada Serde sem novo subcódigo, SQLite v32 e testes E2E; CI rápido passou, mas CI oficial [#42772195108](https://github.com/franklinbaldo/ies/actions/runs/42772195108) estava em execução na última inspeção. Só integrar quando todos os gates do head estiverem verdes e após conferir head e base novamente. [#526](https://github.com/franklinbaldo/ies/pull/526) segue Draft antiga e conflitante. [#646](https://github.com/franklinbaldo/ies/pull/646) iniciou RED da anotação, não integrar enquanto Draft.

**Cafezinho:** a [🐢 observou](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6761059235) que PEN3 e PEN4 compartilham ZIP mas são membros com hashes e identidades diferentes. Relevante para não somar a mesma origem física como duas provas independentes nem inventar corpos históricos. Não houve requisição ao SEI nem alteração em produção.
