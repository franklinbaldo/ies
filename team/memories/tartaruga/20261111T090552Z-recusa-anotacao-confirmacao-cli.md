---
type: AgentMemory
title: "Recusa do botão de anotação também na confirmação"
description: "PR #767 preserva a causa de name repetido; E2E real ampliado para prévia e --confirmar, aguardando CI completo do SHA c562e95."
agent: tartaruga
timestamp: "2026-11-11T09:05:52Z"
tags: [issue718, processo-enviar, anotacao, erros-tipados, e2e, ci, cassette-first]
---

# Para a próxima rodada

**Fato:** [PR #767](https://github.com/franklinbaldo/ies/pull/767) corrige a perda de `ErroAtributoHtml` ao consultar o botão de anotação. No [commit c562e95](https://github.com/franklinbaldo/ies/commit/c562e954031bcc59a85d8a8fd9e7417d16fc74fe) o mesmo E2E com binário real/HTTPS local/anti-IES/SQLite passou a verificar tanto prévia quanto `--confirmar` isoladamente, exigindo zero POST mutador e estado intacto. [CI rápido #42991895487](https://github.com/franklinbaldo/ies/actions/runs/42991895487) PASS; [CI completo #42991999317](https://github.com/franklinbaldo/ies/actions/runs/42991999317) estava em execução ao registrar esta nota.

**Limite:** hashes de cassettes UnB/GO verificados anteriormente (77 HTTP); atributo `name` repetido é hipótese sintética, não confronto independente nem live. A revisão 🦅 de head anterior foi favorável, mas o head novo exige gate e re-review.

**Retomar:** verificar 7/7 no SHA final e sincronizar a branch após esta memória na `main`, sem rebase; encaminhar à 🦅 e integração 🐘/🦡. Próxima ablação livre da #718 apontada pelo 🐘: `processo_acompanhar/fluxo/retirada.rs`, em que `.ok()` perde a causa da releitura após POST. Preserve incerteza e POST único. [Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388) visitado; comentário opcional.
