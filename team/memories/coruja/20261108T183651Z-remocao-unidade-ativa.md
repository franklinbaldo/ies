---
type: AgentMemory
title: "Remoção de anotação depende da unidade ativa"
description: "PR #492 corrige oferta da remoção de anotação da ORIGEM fechada no formulário da DESTINO, com RED/GREEN HTTP e SQLite; integridade das cassettes não é paridade."
agent: coruja
timestamp: "2026-11-08T18:36:51Z"
tags: [processos, anti-ies, sqlite, reconciliacao, anotacoes]
---

# Para retomar

**Fato:** [PR #492](https://github.com/franklinbaldo/ies/pull/492) corrige uma hipótese do anti-IES: ao enviar ORIGEM→DESTINO sem manter aberto, a tela de envio de DESTINO não deve oferecer `chkSinRemoverAnotacoes` da anotação da ORIGEM. O [RED oficial](https://github.com/franklinbaldo/ies/actions/runs/42510006398) falhou na oferta; o [GREEN oficial](https://github.com/franklinbaldo/ies/actions/runs/42510563554) passou 7/7 no head `442f363e`. Regressão testa recusa de POST indevido sem mutação e continuidade DESTINO→ARQUIVO no mesmo SQLite. A PR está aberta para revisão e integração.

**Limite:** a [inspeção de cassettes](https://github.com/franklinbaldo/ies/actions/runs/42510057762) validou 117 interações UFRJ/UnB/GO por hash, não a semântica da transição; live e REA não executados. Não liberar a remoção real. Retomar PR integrada/main atual antes de criar trabalho. No [Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388), a discussão sobre fólio examinado vazio versus página perdida é analogia útil para não confundir anotação ausente com anotação não observada — não é evidência do SEI.
