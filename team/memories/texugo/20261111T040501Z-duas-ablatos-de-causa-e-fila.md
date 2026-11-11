---
type: AgentMemory
title: "Duas ablações de causa integradas e fila de reconciliação"
description: "Merges #747 e #746 com CI 7/7 no head; #752 reconcilia vazamento potencial de bytes e #750 precisa atualizar branch com sobreposição funcional."
agent: texugo
timestamp: "2026-11-11T04:05:01Z"
tags: [antichurn, integracao, ci, causas-tipadas, seguranca, sqlite]
---

# Para retomar

**Fatos verificados:** li `AGENTS.md`, `team/texugo.md`, ciclo cassette-first, cinco memórias recentes e o [Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6786602466). Integrei duas fatias técnicas da [#719](https://github.com/franklinbaldo/ies/issues/719#issuecomment-6786636218):
- [#747](https://github.com/franklinbaldo/ies/pull/747) → [`dcae4a4`](https://github.com/franklinbaldo/ies/commit/dcae4a4e5bbd3798fe951794f336adcf75395e2c), causa `#[source]` nas duas variantes pós-POST/releitura. RED técnico comprovado, CI oficial [7/7](https://github.com/franklinbaldo/ies/actions/runs/42956008168), rápido PASS, head `8e10d07`, `behind=0` e LGTM 🦅.
- [#746](https://github.com/franklinbaldo/ies/pull/746) → [`c07bc09`](https://github.com/franklinbaldo/ies/commit/c07bc09bee07515b1cf6e55640f0b1ac888037e4), `FalhaDaGravacao` e `ErrorKind` existentes preservados no cache. CI oficial [7/7](https://github.com/franklinbaldo/ies/actions/runs/42956227032), rápido PASS no head `ab81306`, LGTM 🦅. A main avançou depois da sincronização apenas na confirmação documental da #747 e em uma memória OKF, sem sobreposição com `src/cache_documento.rs`; dispensei novo merge ritual. RED inicial foi cancelado e não comprovado. `ErrorKind` não conserva o erro IO completo.

**Fila:** [#752](https://github.com/franklinbaldo/ies/pull/752) corrigiu potencial retenção de bytes privados no erro UTF-8 (RED com sentinelas e testes técnicos). Head reconciliado `e4ccb0d`, CI rápido PASS, oficial [#42956733611](https://github.com/franklinbaldo/ies/actions/runs/42956733611) ainda em progresso na última conferência; não integrar sem 7/7 do head. [#750](https://github.com/franklinbaldo/ies/pull/750) tem CI 7/7 no head antigo, mas `behind=31` e sobreposição de arquivos de atribuição na main; [orientei reconciliação mínima](https://github.com/franklinbaldo/ies/pull/750#issuecomment-6786668936). [#526](https://github.com/franklinbaldo/ies/pull/526) segue Draft/conflitante, migração histórica e E2E causal a preservar.

**Três provas:** #747/#746 são ablações técnicas locais com testes Rust e suítes gerais PASS, sem novo E2E específico da CLI ↔ anti-IES ↔ SQLite. Confronto semântico bilateral com cassettes e live NOT_RUN/não aplicável às falhas locais. Nenhuma operação no SEI, segredo, subcódigo ou gate novo. Cafezinho: preservar o diagnóstico sem carregar o payload é uma fronteira substantiva, não burocrática.
