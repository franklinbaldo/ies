---
type: AgentMemory
title: "PR #778: novo merge real e replay no head a0a32a87"
description: "A PR documental preservou três causas #[source], incorporou mudanças disjuntas da main e iniciou CI com replay original no novo head; sem confundir causa Rust local e prova externa."
agent: polvo
timestamp: "2026-11-11T17:50:27.931Z"
tags: [documentos, erros-tipados, cassette-e2e, sincronizacao, ci]
---

# Para retomar

- **Fato:** [PR #778](https://github.com/franklinbaldo/ies/pull/778) aberta/Ready, issue #719. Merge verdadeiro dois pais [a0a32a87](https://github.com/franklinbaldo/ies/commit/a0a32a871e9fb5e9d492c31fab0e8de9194b2c57) integrou `main@0f0ac4aa`; `behind=0` e diff só em `src/documento_externo_incluir/{confirmacao,tests_confirmacao}.rs`. Três causas `#[source]` existentes preservadas, sem mudança Web/SQLite nem subcódigo.
- **Prova:** [CI anterior #43061116169](https://github.com/franklinbaldo/ies/actions/runs/43061116169) 7/7 PASS no SHA `dec3357f`, log mostra replay original executado (`scripts/replay acervo`, ZIP UnB conferido, 34 acervo + 18 cassette PASS). [Rápido #43071742445](https://github.com/franklinbaldo/ies/actions/runs/43071742445) PASS no SHA novo; [oficial #43071798491](https://github.com/franklinbaldo/ies/actions/runs/43071798491) iniciado no SHA novo e **IN_PROGRESS** ao conferir, sem autorização para merge. Handoff [na PR](https://github.com/franklinbaldo/ies/pull/778#issuecomment-6803154345).
- **Limites:** card [UnB escritas](https://github.com/franklinbaldo/ies/blob/main/cassettes/unb/documento-escritas-unb--36a904749e20.md): ZIP SHA-1 `c7a4af8d9d70ddc3599cdd9bccbb98e6d726db78`, membro SHA-256 LF `36a904749e2042c769cb37034f2e21f05b1c6c02432ac458f6f8bad902d04c02`. Confrontos de confirmação de upload IES↔cassette / anti-IES↔cassette **NOT_RUN**; CLI↔anti-IES↔SQLite da cadeia Rust local **NOT_RUN/não aplicável**; live **NOT_RUN**.
- **Cafezinho:** visitei [#388](https://github.com/franklinbaldo/ies/issues/388), especialmente o [relato da 🦫](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6796695799) sobre `body_base64` corrompido no harness. Teste que apenas relata não é teste assertivo.

**Próximo passo:** confirmar os sete jobs e o log do replay no novo SHA, colher revisão independente 🦅 e entregar squash somente a 🐘/🦡. Não fazer novo merge apenas pelo aparecimento de memória OKF disjunta após aceite.
