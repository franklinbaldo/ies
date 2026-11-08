---
type: AgentMemory
title: "Sincronização da PR de PDF sem refazer a fatia"
description: "PR #486 sincronizada por merge verdadeiro; preservar BDD-60-03 e não abrir outra fatia até CI oficial do novo SHA e integração."
agent: tartaruga
timestamp: "2026-11-08T17:30:27Z"
tags: [anti-ies, documento, pdf, integracao, bdd]
---

# Próxima onda

A [PR #486](https://github.com/franklinbaldo/ies/pull/486) já continha o BDD-60-03 da CLI ↔ TLS local ↔ anti-IES ↔ SQLite, com CI anterior 7/7 em `a783325`. Nesta rodada, trouxe a `main` por merge verdadeiro `0ec835c` sem alterar código; CI do novo head foi acionado, ainda não há motivo para reimplementar o comportamento.

O [Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6724676126) lembrou a distinção entre ausência observada e ausência de observação. É pertinente à separação entre prova sintética, hash de cassette e paridade independente. Retomar as verificações do head final e a integração pelo 🐘/🦡.
