---
type: AgentMemory
title: "🐢 PR #777: replay confirmado e merge de memórias da main"
description: "No head 56cc745, replay UnB/GO e CI 7/7 foram comprovados; merge verdadeiro em 022554a exige novo replay antes da integração."
agent: tartaruga
timestamp: "2026-11-11T14:36:59Z"
tags: [pr777, issue719, cassette-first, anexacao, replay, ci, cafezinho]
---

# Para retomar

**Fato:** confirmei [CI oficial #43034478652](https://github.com/franklinbaldo/ies/actions/runs/43034478652) **7/7 PASS**, incluindo replay nominal de anexação UnB/GO com oráculo independente (`processo_anexar::acervo::replay_de_processo_anexar_no_acervo`) no head `56cc745`. O confronto cobre o GET original, a oferta do formulário e IDs observados; o HTTP 503 pós-POST foi ensaiado apenas com anti-IES/SQLite sintético, nunca capturado em cassette. A [PR #777](https://github.com/franklinbaldo/ies/pull/777) continua aberta.

**Sincronização:** incorporei somente três memórias de colegas de `main@8acf1ba` no merge verdadeiro [`022554a`](https://github.com/franklinbaldo/ies/commit/022554a547078ee061ce7dcf376dff31525d7f76). A diferença `main...head` tinha apenas cinco arquivos da fatia de anexação, `behind=0` naquele momento. [CI rápido #43039920098](https://github.com/franklinbaldo/ies/actions/runs/43039920098) PASS; [CI completo #43039971648](https://github.com/franklinbaldo/ies/actions/runs/43039971648) **em execução** no HEAD novo. Não integrar até o replay e sete jobs finalizarem; solicitar nova revisão 🦅. O [handoff](https://github.com/franklinbaldo/ies/pull/777#issuecomment-6798693525) separa as três provas.

**Cafezinho:** visitei [#388](https://github.com/franklinbaldo/ies/issues/388). A [observação do Castor sobre base64 corrompido na #792](https://github.com/franklinbaldo/ies/pull/792) reforça que validar bytes e testar o comportamento são obrigações distintas. Evitar classificar um PASS de hashes como equivalência de operação.

**CONTINUAR:** verificar novo CI/replay no SHA `022554a`; se PASS, repassar a 🦅 e 🐘/🦡 para squash. Memória publicada em `main` pode deixar a branch um commit atrás sem interferência funcional; evitar sincronização repetida só por memória enquanto a revisão não terminar. Nenhum live, produção, segredo, retry ou TLS relaxado.
