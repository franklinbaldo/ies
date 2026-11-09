---
type: AgentMemory
title: "BDD-60-01/02: leitura e versão com processo explícito na CLI real"
description: "PR #527 acrescentou E2E da CLI real com processo/identidade e foto histórica; CI completo 7/7 no SHA ba99806, cassettes UnB 42 interações conferidas só estruturalmente."
agent: tartaruga
timestamp: "2026-11-09T01:50:16Z"
tags: [anti-ies, documento, bdd, versoes, cli, sqlite, cassette]
---

# Para retomar

**Fato:** [PR #527](https://github.com/franklinbaldo/ies/pull/527) está Ready com dois testes de BDD-60-01/02 no binário `ies` contra HTTPS local → anti-IES → SQLite. A leitura com `--processo` preserva identidade e conteúdo; `--versao 1 --saida` fotografa `DOC.v1.html` e recusa a edição local divergente na segunda leitura. A implementação existente já atendia aos testes: **não houve RED funcional**, apenas uma falha inicial de Rustfmt corrigida. [CI oficial #42547170177](https://github.com/franklinbaldo/ies/actions/runs/42547170177) **7/7** no head `ba99806b8a33138d3ee0c5a91ee24e684e2b8c95`, e [rápido #42547164682](https://github.com/franklinbaldo/ies/actions/runs/42547164682) passou. Integração pendente de 🦅/🐘/🦡, sem merge pela Tartaruga.

**Cassette-first:** [Actions #42547068781](https://github.com/franklinbaldo/ies/actions/runs/42547068781) verificou SHA-1/SHA-256 e estrutura de dois cards UnB (15 + 27 = **42** interações HTTP); marcador temporário removido. **CLI↔anti-IES PASS sintético; ambos↔cassette semântico NOT_RUN; live NOT_RUN; REA NOT_RUN.** Não confundir o hash com paridade do comportamento.

**Continuidade:** acompanhar integração da #527 e evitar duplicar a cobertura dos [BDD #60](https://github.com/franklinbaldo/ies/issues/60). O [Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388) discutiu que duas sequências podem produzir a mesma projeção; isso inspira buscar testes independentes, mas é metáfora, não dado observado do SEI.
