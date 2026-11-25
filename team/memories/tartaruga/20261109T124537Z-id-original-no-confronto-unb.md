---
type: AgentMemory
title: "Confronto UnB: a identidade do GET original precisa atravessar o SQLite"
description: "PR #570 substitui ID documental arbitrário por id_documento da cassette no teste anti-IES; RED 503, GREEN 7/7 no SHA 471e270d, paridade parcial sem live."
agent: tartaruga
timestamp: "2026-11-09T12:45:37Z"
tags: [bdd60, cassette, identidade, anti-ies, sqlite, http, ci]
---

# Para retomar

**Fato:** na [PR #570](https://github.com/franklinbaldo/ies/pull/570), o teste diferencial de conteúdo UnB já comparava texto e tipo, mas o anti-IES publicava o ID arbitrário `42`. O GET original da cassette contém `id_documento`; o teste agora extrai esse valor e o preserva no SQLite e nas requisições HTTP, recusando múltiplos IDs distintos. [RED #42618693308](https://github.com/franklinbaldo/ies/actions/runs/42618693308): 0 PASS/1 FAIL, HTTP 503 porque o ID observado não constava do banco. [GREEN #42619046283](https://github.com/franklinbaldo/ies/actions/runs/42619046283): 7/7 jobs PASS no head `471e270d`; replays de conteúdo e histórico UnB PASS, [rápido #42619021624](https://github.com/franklinbaldo/ies/actions/runs/42619021624) PASS.

**Limites:** confronto com cassette **parcial**; processo, NUP e revisão no simulador continuam hipóteses, sem novo E2E específico do binário CLI. Regressões E2E anteriores passaram na suíte. Treinamento e REA NOT_RUN. Nada produtivo mudou, nem TLS, subcódigos ou credenciais.

**Cafezinho:** visitei [#388](https://github.com/franklinbaldo/ies/issues/388), inclusive a distinção entre ausência de observação e negação observada. Aqui há uma distinção irmã: o documento pode ter corpo correto e, ainda assim, receber identidade inventada. A conversa é inspiração, não evidência de SEI.

**Continuar:** [handoff #60](https://github.com/franklinbaldo/ies/issues/60#issuecomment-6740366801); PR #570 Ready para revisão 🦅 e integração 🐘/🦡. A `main` avançou por memórias documentais após a validação, a conciliar na integração. Manter #60 aberta para lacunas externas demonstráveis.
