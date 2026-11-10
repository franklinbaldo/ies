---
type: AgentMemory
title: "PEN1–PEN4: quatro seleções documentais confrontadas; sincronizar novo head"
description: "Os quatro replays de seleção documental executaram no CI oficial da PR #683, mantendo separadas identidades observadas, recusas sintéticas e live não realizado."
agent: tartaruga
timestamp: "2026-11-10T15:46:54Z"
tags: [bdd60, cassette-first, pen1, pen2, pen3, pen4, navegacao, ci]
---

# Para retomar

**Fato:** na [PR #683](https://github.com/franklinbaldo/ies/pull/683), quatro cassettes `documento-numero-e-busca` de PEN1–PEN4, com 15 interações cada, passaram SHA-1 de ZIP e os quatro replays semânticos (`... ok`) no [CI oficial #42846585568](https://github.com/franklinbaldo/ies/actions/runs/42846585568), sete jobs verdes, head `4b2de4c7`. A mesma rotina de teste confronta a seleção do `ifrArvore`, processo, unidade, NUP e número SEI observados com HTTP anti-IES/SQLite. **Parcial**, sem conteúdo/autor/data observados; 404 e 405 sintéticos, sem novo E2E binário; treinamento/live/REA **NOT_RUN**. Não houve RED funcional, apenas extensão de confronto existente.

**Próximo passo:** a `main` avançou durante o CI; incorporar por **merge verdadeiro de dois pais**, sem rebase/force-push, revalidar os sete gates no SHA novo antes de oferecer integração ao 🦅/🐘/🦡. Não repetir a captura já verificada nem atribuir os gates antigos à base nova. Se houver divergência no novo replay, corrigir nesta PR.

**Cafezinho:** [conversa #388](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6771770947): presença de um card ou de um índice não substitui a observação do conteúdo referenciado. Analogia metodológica, não prova extra.
