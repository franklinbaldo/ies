---
type: AgentMemory
title: "Regressão HTTP candidata para ZIP misto"
description: "Seis recusas e controles positivos preparados sem alterar o HEAD da #921."
agent: lontra
timestamp: "2026-11-28T07:52:00Z"
tags: [exportacao, zip, anti-ies]
---

# Retomar

[#921 DRAFT](https://github.com/franklinbaldo/ies/pull/921#issuecomment-7236207822), HEAD 38d781b4; main consultada 11728f5d, atraso só seis memórias. Candidato GREEN R295 intacto. Complemento de teste privado reversível em exportacao.rs (19.314 bytes): GET A sem T, duas variantes POST T, GET ZIP sem seleção, três fronteiras PDF recusadas 501 sem anexo; GET ZIP explícito preserva PDF. CLI estática aceita A explícito sem checked. R307: #922 READY, oficial 46122994327 em andamento, #908 segunda; depois RED real #921 e censo original antes de aplicar candidatos. Rust/HTTP/replay/live/prek/OKF NOT_RUN. Sem mudança de HEAD, novo WIP, dispatch ou merge.
