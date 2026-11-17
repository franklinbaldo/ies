---
type: AgentMemory
title: "Ablação do cache: duas reconciliações e a fronteira do ErrorKind"
description: "PR #746 voltou a behind=0 em dois merges verdadeiros; CI 7/7 no SHA intermediário e novo CI disparado no SHA final. ErrorKind conserva categoria, não cadeia completa."
agent: castor
timestamp: "2026-11-11T04:00:18Z"
tags: [ies, castor, rust, antipattern, cache, ci, erro-tipado, merge]
---

# Para retomar

**Fatos:** li as instruções e memórias, conferi a [PR #746](https://github.com/franklinbaldo/ies/pull/746) e a revisão 🦅. Incorporei a `main` duas vezes por merge verdadeiro de dois pais: [`ed9db7b`](https://github.com/franklinbaldo/ies/commit/ed9db7bbdd76aa61e8f964a1c527c9c1feb3dbcb), depois [`ab81306`](https://github.com/franklinbaldo/ies/commit/ab81306bd6d45d32ffde1f09f3e067ff4ea54333), sem rebase/force-push. O blob `src/cache_documento.rs` na `main` não mudou nessas reconciliações, e o diff da PR segue só esse arquivo. [CI oficial #42955887531](https://github.com/franklinbaldo/ies/actions/runs/42955887531) **7/7 PASS** em `ed9db7b`; [CI rápido #42956215850](https://github.com/franklinbaldo/ies/actions/runs/42956215850) PASS e [CI oficial #42956227032](https://github.com/franklinbaldo/ies/actions/runs/42956227032) em execução na última conferência para `ab81306`. Revisão 🦅 foi favorável tecnicamente, condicionada ao head reconciliado.

**Limite:** `ArquivoDoCacheInacessivel(io::ErrorKind)` mantém a categoria `NotADirectory`, não `io::Error` inteiro ou `source()`. O RED inicial foi cancelado antes dos testes; não alegar RED funcional. Teste técnico local não é novo E2E CLI/anti-IES; cassette do SEI não é aplicável a esta falha local; live NOT_RUN.

**CONTINUAR:** conferir os sete jobs do SHA novo e a base antes da integração 🐘/🦡, mantendo a [#719](https://github.com/franklinbaldo/ies/issues/719) aberta. Na `src/configuracao.rs`, há possíveis traduções de falhas de E/S para categorias genéricas; **hipótese para triagem posterior**, não defeito comprovado nem convite para gerar wrappers. No [Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6786602466), conversamos sobre preservar diagnósticos sem carregar bytes privados.