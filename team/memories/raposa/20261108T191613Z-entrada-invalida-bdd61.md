---
type: AgentMemory
title: "BDD-61-03: validação de entrada antes da sessão"
description: "Teste de CLI real para termo vazio e controles incluído na #489; CI no novo head ainda não verificado, v13 pendente."
agent: raposa
timestamp: "2026-11-08T19:16:13Z"
tags: [cli, bdd, anti-ies, testes, integracao]
---

# Retomar na próxima rodada

Fato: acrescentei na [PR #489](https://github.com/franklinbaldo/ies/pull/489), commit [4203b06](https://github.com/franklinbaldo/ies/commit/4203b061e8c7c953f778d4672bd61bd2defbf996), teste do binário real recusando termos vazios, LF e tab com saída 2, antes da configuração/rede. O teste novo **ainda não foi validado pelo CI** nesta rodada; o 7/7 anterior pertence a outro SHA.

Fato: a [#487](https://github.com/franklinbaldo/ies/pull/487) ainda estava Ready e aberta; pela decisão antichurn, não conciliar #489 duas vezes. Quando #487 integrar, efetuar merge verdadeiro da main, preservar migrações 0012→0013→0014, corrigir assertivas e rodar CI 7/7 no head reconciliado. Não tratar hashes das cassettes como confronto semântico. A [conversa do Cafezinho](https://github.com/franklinbaldo/ies/issues/388) ressalta que ausência observada é diferente de ausência de observação (metáfora, não teste).
