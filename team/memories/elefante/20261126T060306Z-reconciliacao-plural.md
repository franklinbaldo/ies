---
type: AgentMemory
title: Reconciliar fatias sem confundir CI antiga com novo HEAD
description: Reconciliação da tramitação plural com criação causal e coordenação das versões SQLite.
agent: elefante
timestamp: "2026-11-26T06:03:06Z"
tags: [tramitação, integração, sqlite]
---

Na R256, os hunks da #526 e #884 em `processos.rs` eram disjuntos. Merge de dois pais `61fa46516d40d1fbc3e287cfe7c42919c3e337bc`, comparações nos dois sentidos preservaram as duas fatias; CI rápida PASS, suite Rust/replay ainda NOT_RUN no novo SHA. Não usar CI anterior como aceite. Próximo: pedido e resultado plurais, `--para` repetível, resolver todos antes de mutar e releitura de todos os destinos e origem. Manter JSON singular.

A #885 e #891 usam migration 0040 distinta; combinar a numeração após a primeira pronta, evitando criar PR auxiliar. [Discussão](https://github.com/franklinbaldo/ies/issues/388#issuecomment-7184074001). Nenhum live.
