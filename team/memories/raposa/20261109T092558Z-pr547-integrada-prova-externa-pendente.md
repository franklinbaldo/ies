---
type: AgentMemory
title: "PR #547 integrada: não refazer BDD sintético, distinguir prova de POST"
description: "Confirmação e remoção via CLI real chegaram à main; CI oficial verde no head original, mas cassette POST e live ainda pendentes."
agent: raposa
timestamp: "2026-11-09T09:25:58Z"
tags: [cli, atribuicao, bdd, anti-ies, integracao, cassette, live]
---

# Retomada

**Fato:** a [PR #547](https://github.com/franklinbaldo/ies/pull/547) foi mesclada na `main` pelo [merge 2f8c2fed](https://github.com/franklinbaldo/ies/commit/2f8c2fed6d2783d82ad5d3a93a8ae649c05e18f7). A CLI real confirmou atribuição e remoção no anti-IES/SQLite no [CI 7/7 do head original](https://github.com/franklinbaldo/ies/actions/runs/42586065139); um POST aplicado localmente com resposta 403 conclui com saída 3. A [issue #264](https://github.com/franklinbaldo/ies/issues/264#issuecomment-6736601793) contém o handoff pós-merge.

**Não extrapolar:** o confronto de 28 formulários originais da #539 é apenas GET; efeitos POST/releitura da cassette e execução live permanecem **NOT_RUN**. A autorização de escrita em treinamento está na #264, mas não foi utilizada nesta rodada; nenhuma requisição externa. A `main` avançou enquanto a branch antiga era sincronizada: `c22895e` é um merge redundante na branch após o fechamento da PR, **não** o commit integrado. Não retomar essa PR fechada, não refazer o E2E já verde.

**Próximo passo:** partir da `main` atual; observar efeito original POST quando a cassette o registrar ou avançar ensaio seguro autorizado em treinamento, com prévia, uma confirmação, releitura, uma remoção e relatório de resíduo; nenhuma repetição em efeito incerto. A visita ao [Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388) reforçou a distinção narrativa entre a mensagem que anuncia uma ação e a testemunha que comprova seu efeito, sem converter ficção em evidência SEI.
