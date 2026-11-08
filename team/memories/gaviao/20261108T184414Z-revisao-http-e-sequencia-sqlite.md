---
type: AgentMemory
title: "Revisão #493/#492/#491/#487: RED HTTP e sequência SQLite"
description: "Quatro pareceres por head: #493 tem teste HTTP RED e Rustfmt falho; #492 possui CI 7/7 em regressão técnica; #491 prévia CLI verde aguarda v13/v14; #487 exige conciliar v12 e revalidar."
agent: gaviao
timestamp: "2026-11-08T18:44:14Z"
tags: [revisao, bdd, anti-ies, sqlite, ci, cassettes]
---

# Retomar na próxima rodada

**Fato:** em [#493](https://github.com/franklinbaldo/ies/pull/493#issuecomment-6725571331), head `c05c7ea` trouxe teste HTTP de identidade duplicada, mas sem correção de produção; CI rápido falhou em Rustfmt. Em [#492](https://github.com/franklinbaldo/ies/pull/492#issuecomment-6725571640), head `442f363`, regressão HTTP sobre anotação da ORIGEM fechada está verde 7/7, mas não é novo E2E CLI integral. Em [#491](https://github.com/franklinbaldo/ies/pull/491#issuecomment-6725571941), `2472f89` tem BDD-258-01 sintético 7/7, porém o registrador salta 0012→0015: aguardar #487 v13 e #489 v14 antes do merge. Em [#487](https://github.com/franklinbaldo/ies/pull/487#issuecomment-6725572267), `a00104a` ainda não inclui a 0012 PDF da main nem tem CI 7/7 no próprio SHA.

**Disciplina:** comparar always head e CI exatos; cassette validada por hash não é confronto semântico e não bloqueia implementação. As inspeções realizadas por outros não são REA. Próxima rodada: reexaminar somente heads novos e a sequência de migrações.

**Cafezinho:** a analogia da [pausa escrita versus página faltante](https://github.com/franklinbaldo/ies/issues/388) ajuda a lembrar que ausência observada não é dado desconhecido; é inspiração, não evidência do SEI.
