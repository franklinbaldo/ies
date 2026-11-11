---
type: AgentMemory
title: "Cache privado: a causa atravessa os dois encaixes"
description: "PR #746 preserva ErrorKind e FalhaDaGravacao em gravar_em, com testes técnicos Unix e CI oficial 7/7; execução RED funcional foi cancelada antes dos testes."
agent: castor
timestamp: "2026-11-11T03:00:42Z"
tags: [ies, castor, rust, cache, antipattern, erros-tipados, ci]
---

# Ponto de retomada

**Fatos:** [PR #746](https://github.com/franklinbaldo/ies/pull/746), [issue #719](https://github.com/franklinbaldo/ies/issues/719). Dois `Err(_)` em `cache_documento::gravar_em` descartavam, respectivamente, `io::ErrorKind` e `FalhaDaGravacao`; a mudança os encaminha a variantes já existentes, com testes Unix de `NotADirectory` e diretório inseguro. Branch reconciliada por merge verdadeiro `89f0b023`, `behind=0` ao concluir. [CI completo #42949283012](https://github.com/franklinbaldo/ies/actions/runs/42949283012) **7/7 PASS** e [rápido #42949266676](https://github.com/franklinbaldo/ies/actions/runs/42949266676) PASS no head exato. O run de testes antes da correção foi **cancelado antes da suíte Rust**: não declarar RED funcional executado. Não houve CLI↔anti-IES↔SQLite específico, cassette não se aplica ao erro de filesystem e live não foi executado.

**CONTINUAR:** revisão independente 🦅 e integração 🐘/🦡 da #746, sem mesclar como Castor. A #719 permanece aberta para outras traduções de erro, sem disputa com ramos alheios. Visitei o [Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6785681674): o mesmo motivo pode ser preservado na peça inferior e perdido no encaixe do chamador. Nenhum novo subcódigo, credencial ou acesso ao SEI externo.
