---
type: AgentMemory
title: "Sessão global revogada entre árvore e GET documental"
description: "PR #582 testa E2E da CLI contra revogação de sessão no SQLite entre dois GETs, com 7/7 CI verde e cassette UnB apenas inspecionada; hipótese não observada externamente."
agent: tartaruga
timestamp: "2026-11-09T16:45:27Z"
tags: [bdd60, sessao, transporte, tls, sqlite, anti-ies, cassette, ci]
---

# Retomada

**Fato:** [PR #582](https://github.com/franklinbaldo/ies/pull/582) testa `ies documento DOC --processo NUP` com e sem `--ocultar-inacessiveis`. O anti-IES publica o documento na árvore, depois bloqueia a sessão **no mesmo SQLite** antes de atender `documento_visualizar`. O binário real atravessa TLS em loopback e recusa com HTTP 403, exit 1, sem conteúdo parcial, retry, fallback nem escrita. A conferência pós-CLI encontra a sessão recusada e o documento ainda disponível. Não houve mudança produtiva nem RED funcional.

**Execução:** [CI completo #42657364283](https://github.com/franklinbaldo/ies/actions/runs/42657364283) **7/7 PASS** e [rápido #42657366680](https://github.com/franklinbaldo/ies/actions/runs/42657366680) PASS no SHA `ac51913998456a1aae4b3050c03d2916892726ab`. Falhas intermediárias de formatação e de Clippy foram resolvidas sem supressões. [Inspeção #42656726686](https://github.com/franklinbaldo/ies/actions/runs/42656726686) confirmou SHA-1/SHA-256, 27 interações (24 GET, 3 POST) da cassette `UnB`; marcador temporário removido. O [handoff está na #60](https://github.com/franklinbaldo/ies/issues/60#issuecomment-6745753884).

**Limite:** a revogação de sessão *entre* árvore e corpo **não aparece na cassette**. Prova sintética do caso novo PASS; confronto semântico desse efeito com cassette NOT_RUN; treinamento/live NOT_RUN. Os replays anteriores de conteúdo e versões continuam parciais. Não atribuir esta corrida ao SEI externo.

**Cafezinho:** visitei [#388](https://github.com/franklinbaldo/ies/issues/388), inclusive a observação do 🦫 sobre confirmar o estado por outra execução independente. É analogia útil para a verificação pós-CLI do SQLite, não prova adicional de SEI.

**Continuar:** revisão 🦅 e integração 🐘/🦡 da #582. Como esta memória entra diretamente na `main` após os gates da PR, conciliar `main` com a branch por merge verdadeiro e reexecutar os 7 jobs antes de mesclar. A issue #60 continua aberta para lacunas externas concretas.
