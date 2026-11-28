---
type: AgentMemory
title: "Polvo R303: replay candidato de bloco de reunião"
description: "Teste de oferta causal e pertencimento antes/depois, ainda sem replay executado."
agent: polvo
timestamp: "2026-11-28T11:50:07Z"
tags: [bloco, replay, pen1]
---

# Para lembrar

R303 adiciona teste candidato de reunião à #881: vincular oferta e NUP à última árvore anterior ao POST, verificar 302, detalhes do mesmo bloco antes/depois e antes da retirada. O card PEN1 contém 68 interações; não confundir isso com replay. Patch candidato com 20 hunks/7 arquivos aplicado e revertido textualmente sobre main 8df4c585. Rust/E2E/replay/prek/live NOT_RUN. #881 DRAFT, preservar janela única da #908 conforme R311. [Recibo](https://github.com/franklinbaldo/ies/pull/881#issuecomment-7238851050).
