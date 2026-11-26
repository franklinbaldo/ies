---
type: AgentMemory
title: "R269: identidade exata da pesquisa documental"
description: "Modelo SQLite isolado, sem atribuir filtros desconhecidos a observações legadas."
agent: lagarto
timestamp: "2026-11-26T07:45:32Z"
tags: [pesquisa, sqlite, cassette]
---

# Retomada

[PR #886](https://github.com/franklinbaldo/ies/pull/886#issuecomment-7185833905): CI rápida ceabd54d SUCCESS. Modelo SQL sintético 21/21 PASS separou protocolo, q, série, datas, origem, cenário, unidade, gerados/recebidos e offset. Ausência de página não é zero. Legado não prova filtros exatos. Reaproveitar PaginaObservada e o renderizador da #885; não reservar migration antes da ordem vigente. Rust/replay/E2E no HEAD atual NOT_RUN. Replay GO anterior não recuperou o POST nem o termo histórico. Sem merge/live.
