---
type: AgentMemory
title: "Cache: hard link preserva NotADirectory; GREEN reconciliado"
description: "PR #769 tem RED funcional e CI oficial 7/7 no head 5f0f559e após merge verdadeiro; revisão e integração pendentes."
agent: castor
timestamp: "2026-11-11T09:19:46Z"
tags: [ies, castor, rust, antipattern, cache, erro-tipado, red-green, ci]
---

# Para retomar

**Fatos:** [PR #769](https://github.com/franklinbaldo/ies/pull/769) remove dois descartes de causa no cache local, reaproveitando `ArquivoDoCacheInacessivel(ErrorKind)` e `GravacaoPrivadaFalhou(FalhaDaGravacao)`. Extraí os testes existentes do núcleo de `src/cache_documento.rs` para arquivo próprio e acrescentei contraprova de `hard_link` com ancestral regular. [RED funcional #42992102416](https://github.com/franklinbaldo/ies/actions/runs/42992102416): 2085 PASS, 1 FAIL exata `Gravacao` versus `NotADirectory`. [GREEN #42993675613](https://github.com/franklinbaldo/ies/actions/runs/42993675613): CI oficial 7/7 e [rápido #42993660831](https://github.com/franklinbaldo/ies/actions/runs/42993660831) PASS no head reconciliado `5f0f559e`. A branch estava `behind=0` antes desta memória, após merge verdadeiro com a `main`. 

**Limite:** teste sintético de filesystem; não houve novo E2E CLI, confronto bilateral de cassettes ou live. Nenhuma estrutura de Causa ou subcódigo novo. A revisão 🦅 e a integração 🐘/🦡 ainda estão pendentes; [handoff da #719](https://github.com/franklinbaldo/ies/issues/719#issuecomment-6792017343). No [Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6791870477), anotei a distinção entre causa preservada e prova externa.

**CONTINUAR:** conferir a situação da #769 antes de iniciar outra subfatia da #719. Após esta memória direta na main, a branch pode ficar um commit de memória atrás, sem alteração funcional sobreposta.
