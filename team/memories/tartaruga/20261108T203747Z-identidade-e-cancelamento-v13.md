---
type: AgentMemory
title: "Identidade e cancelamento v13 conciliados na PR #493"
description: "Merge verdadeiro restaurou compatibilidade da recusa de documento ambíguo com cancelamento v13; CI oficial 7/7 no head be87662."
agent: tartaruga
timestamp: "2026-11-08T20:37:47Z"
tags: [navegacao, anti-ies, documento, cancelamento, integracao]
---

# Próxima onda

Fato: a [PR #493](https://github.com/franklinbaldo/ies/pull/493) estava 61 commits atrás da main e conflitou com o cancelamento documental/SQLite v13. O [merge verdadeiro `be87662`](https://github.com/franklinbaldo/ies/commit/be87662b09968e5012ce8ecd185212aa69fb0e39) preservou o cancelamento e a recusa de `id_documento` duplicado sem fallback, além dos testes HTTP e BDD-60-04 da CLI real. O [CI oficial](https://github.com/franklinbaldo/ies/actions/runs/42520888126) aprovou 7/7 e o rápido também passou. PR Ready e sem conflito na conferência. Não repetir sincronização de memórias como trabalho funcional; 🐘/🦡 podem integrar a fatia. A issue #60 fica aberta para BDD pendente.

Cassette-first: cards UnB relidos; inspeção anterior de hashes é estrutural, **não** paridade semântica. Treinamento e REA não executados. No [Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388), a metáfora de operações não comutativas (prensa e dobra, fuzz e compressor) lembra por que duas alterações precisam de teste conjunto após um merge, mas não é evidência do SEI.
