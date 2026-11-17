---
type: AgentMemory
title: "Árvore com PDF: seleção contraditória não autoriza fallback"
description: "Na PR #543, RED provou HTTP 200 indevido para IDs conflitantes na árvore mista; correção mínima recusa 400 com CI completo 7/7, cassettes apenas verificadas estruturalmente."
agent: tartaruga
timestamp: "2026-11-09T06:19:29Z"
tags: [bdd60, navegacao, anti-ies, http, sqlite, cassette]
---

# Para retomar

**Fato:** na [PR #543](https://github.com/franklinbaldo/ies/pull/543), o [RED #42571040141](https://github.com/franklinbaldo/ies/actions/runs/42571040141) mostrou 200 com conteúdo do PDF quando `id_documento` e `id_protocolo` divergiam. A árvore mista passou a usar os validadores existentes e recusar 400. [CI completo #42571866525](https://github.com/franklinbaldo/ies/actions/runs/42571866525) 7/7 e rápido PASS no SHA `76f9dd4`; PR Ready para 🦅/🐘/🦡. Teste HTTP/SQLite sintético, sem E2E novo do subcaso pela CLI; a suíte de CLI existente passou.

**Limite:** [inspeção UnB/GO #42571023803](https://github.com/franklinbaldo/ies/actions/runs/42571023803) verificou ZIPs, hashes e 30 HTTP; **ambos↔cassette semântico NOT_RUN**, treinamento NOT_RUN. Não atribuir a seleção contraditória ao SEI real. Marker temporário removido. Não reimplementar o caso já coberto; acompanhar integração e procurar só lacuna real da #60.

**Cafezinho:** [Castor observou analogia semelhante](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6733571131): identidade repetida em representações diferentes só é testemunha se for **coerente entre elas**. Inspirou a distinção entre o rótulo e o vínculo, mas não acrescenta prova externa ao teste.
