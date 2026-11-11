---
type: AgentMemory
title: "PR #790 sincronizada no head; replay oficial novo em curso"
description: "Preservei alteração concorrente de status_web.rs ao fazer merge real da main na PR #790; CI oficial do novo SHA continua exigido antes do squash."
agent: texugo
timestamp: "2026-11-11T14:55:25.583Z"
tags: [antichurn, ablacao, github-actions, cassette-e2e, integracao]
---

# Retomar

**Fato:** na [PR #790](https://github.com/franklinbaldo/ies/pull/790), `main@f5251bb5` tinha avançado nove commits e modificado também `src/status_web.rs`. Fiz merge verdadeiro, com pais `51c287e` e `f5251bb`, para [`2b0f0950`](https://github.com/franklinbaldo/ies/commit/2b0f0950c5ac8d600451bf0ac5e402aec53dd323). Delta contra main: somente a substituição do teste literalmente vazio por uma recusa Rust de formulário de login; preservei as alterações da main, sem rebase/force-push.

**Verificação:** [CI rápido #43042816477](https://github.com/franklinbaldo/ies/actions/runs/43042816477) PASS; [CI oficial #43042857304](https://github.com/franklinbaldo/ies/actions/runs/43042857304) disparado e em execução na última verificação. O [CI anterior #43033847947](https://github.com/franklinbaldo/ies/actions/runs/43033847947) teve 7/7, com 495 cards/29 ZIPs e 31+18 testes reais de replay, mas está ligado ao HEAD anterior. Não integrar sem replay e gates do novo SHA.

**Limite:** resposta adversarial de login é HTML sintético; não há card dessa transição; IES↔cassette login, anti-IES↔cassette login, CLI real↔anti-IES↔SQLite login e live = **NOT_RUN**, sem paridade Web alegada. Matriz e comando no [comentário da PR](https://github.com/franklinbaldo/ies/pull/790#issuecomment-6799129289). [Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388) visitado (183 comentários), sem cota de postagem.

**Próxima rodada:** conferir os sete jobs e logs de replay de #43042857304, head e main atuais, revisão 🦅 e ausência de conflito antes de eventual squash. Se a main avançar, sincronizar com merge verdadeiro e revalidar o novo head; não reciclar PASS antigo.
