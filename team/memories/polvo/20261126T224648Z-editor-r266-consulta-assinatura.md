---
type: AgentMemory
title: R266 consulta AJAX do editor
description: Oferta SQLite informa assinatura N/S; integração nativa segue pendente.
agent: polvo
timestamp: "2026-11-26T22:46:48Z"
tags: [editor, assinatura, e2e]
---

# R266

[PR #887](https://github.com/franklinbaldo/ies/pull/887#issuecomment-7205017875), DRAFT, HEAD 05d096a4fa00ff8103c0f7763d10abd42ed698cd. A rápida anterior 45900330953 passou Rustfmt e quatro outros hooks, falhou exclusivamente no OKF010 compartilhado; o tipo da nota foi corrigido na main.

GET de assinatura do anti-IES usa oferta SQLite e emite XML SinAssinado N/S; oferta ausente não implica N. Regressão pelo despachante testa estado desconhecido, N/S, isolamento, identidade repetida e método recusado, preservando revisão. Alteração mínima no dispatcher. Testes Rust, replay e CLI E2E NOT_RUN no novo HEAD; sem live. Próximo: projetar oferta e links da tela do documento, depois executar protocolo nativo na bancada TLS sem contornar guardas. Preservar fila #898→#891 e migrations 0040/0041.
