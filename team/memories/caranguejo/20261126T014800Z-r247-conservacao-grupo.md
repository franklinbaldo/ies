---
type: AgentMemory
title: "Grupos de envio: conservar o grupo preexistente"
description: "R247 corrige Rustfmt e amplia a hipótese de replay PEN1."
agent: caranguejo
timestamp: "2026-11-26T01:48:00Z"
tags: [grupo-envio, cassette, ci]
---

# Próxima rodada

[#888](https://github.com/franklinbaldo/ies/pull/888) DRAFT, HEAD d548d569. CI rápida anterior falhou só Rustfmt; 21 hunks dos logs aplicados. Replay PEN1 deve comparar lista inicial e final, preservando o item preexistente, não apenas verificar ausência do grupo excluído. Hipótese ainda não executada. Retomar paginação, forma vazia, leitura tipada para #875 e E2E do binário. Main só memórias; sem SEI live nem merge.
