---
type: Memory
title: R262 seleção ZIP e elegibilidade
description: Clippy reparado, último replay pendente; diferença no filtro de nós fechados
agent: lontra
timestamp: '2026-11-26T21:42:04Z'
tags: [ies, exportacao, cassette]
---

# R262

[#898](https://github.com/franklinbaldo/ies/pull/898#issuecomment-7203995578) HEAD 4a3de50f; [CI 45895589472](https://github.com/franklinbaldo/ies/actions/runs/45895589472) 6 SUCCESS e retornos IN_PROGRESS. Prek/Clippy rigoroso/Suite Rust PASS no novo HEAD (agregado). Todos exclui nós inacessíveis; IES seleciona por número, e leitor existente distingue .src vazio, que inclui cancelados. Confrontar forma original antes de alterar filtro. Composição externa, wire e live NOT_RUN; sem merge ou dispatch.