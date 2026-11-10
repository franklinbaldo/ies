---
type: AgentMemory
title: "BDD-257-03: efeito material não elimina incerteza causal"
description: "PR #661 exercita CLI real, HTTPS local, anti-IES e SQLite com POST único persistido e 503 sintético na primeira releitura; saída 3, sem retry, CI 7/7 no head final."
agent: castor
timestamp: "2026-11-10T11:49:31Z"
tags: [ies, anti-ies, comentarios, bdd, rust, sqlite, cassettes, ci]
---

# Para retomar

**Fato:** na [PR #661](https://github.com/franklinbaldo/ies/pull/661), acrescentei regressão sintética do BDD-257-03 na CLI Rust real sobre HTTPS loopback, anti-IES e SQLite v33. O túnel deixa o único POST persistir e responde HTTP 503 na primeira releitura; a CLI devolve **saída 3**, `codigo=efeito_incerto`, `retentavel=false` e instrui a consultar comentários e não repetir `--confirmar`. Outra CLI independente comprova os dois comentários e texto exato, sem retry. Nenhum novo subcódigo; nenhum acesso ao SEI externo.

**Gates:** [CI oficial #42809454227](https://github.com/franklinbaldo/ies/actions/runs/42809454227) teve suíte Rust PASS, mas Clippy acusou 112 linhas na função de teste; extraí `atender_conexao_tunel` sem suprimir lint. No head `653068bbfdb4914c734777bce2334d5cf48f2228`, [CI oficial #42809863043](https://github.com/franklinbaldo/ies/actions/runs/42809863043) **7/7 PASS** e [CI rápido #42809855733](https://github.com/franklinbaldo/ies/actions/runs/42809855733) PASS. O BDD-257-03 não teve RED funcional novo, pois a saída 3 já existia; não chamar falha Clippy de RED comportamental.

**Provas:** IES ↔ anti-IES ↔ SQLite **PASS sintético**; integridade de 76 interações UnB/GO [PASS anterior](https://github.com/franklinbaldo/ies/actions/runs/42786966461); replay semântico bilateral **NOT_RUN**, treinamento/live **NOT_RUN**. 503 é hipótese, não capture. Discussões sobre causalidade no [Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6766530146) recordam que o estado posterior não autentica a certeza da tentativa.

**CONTINUAR:** revisão 🦅 e integração 🐘/🦡 da #661, que estava aberta e pronta; a `main` avançou com memória do 🦅 antes desta nota, devendo-se conferir a comparação e fazer merge verdadeiro final sem force-push antes da integração. Depois, tratar lista inicial vazia/302 e comparação semântica externa sem inventar resultado.
