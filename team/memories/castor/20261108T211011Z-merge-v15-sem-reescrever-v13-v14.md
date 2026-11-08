---
type: AgentMemory
title: "Merge verdadeiro preserva v13, v14 e v15 em peça única"
description: "Seis conflitos reais resolvidos na PR #491 sem perder cancelamento, busca negativa ou relacionamentos; CI completo 7/7 no head conciliado."
agent: castor
timestamp: "2026-11-08T21:10:11Z"
tags: [rust, sqlite, anti-ies, bdd, merge, ci]
---

# Próximo encaixe

**Fato:** a [PR #491](https://github.com/franklinbaldo/ies/pull/491) tinha seis conflitos com a `main` depois de integrar #487 (SQLite v13) e #489 (v14). O [Actions #42523473919](https://github.com/franklinbaldo/ies/actions/runs/42523473919) executou merge verdadeiro no commit `0c544fa2`, combinando os registradores, rotas e asserts sem escolher cegamente um dos lados. O workflow temporário foi removido. O [CI oficial #42523534025](https://github.com/franklinbaldo/ies/actions/runs/42523534025) aprovou **7/7** no head `38be715d`. Sequência: SQL v12→v13→v14→v15.

**Evidência:** BDD-258-01 tem **PASS sintético** (CLI real/TLS local/anti-IES/SQLite); os cards PEN1 já tiveram hashes conferidos, mas confronto semântico independente e treinamento seguem **NOT_RUN**. REA não foi executada. A PR está Ready para revisão 🦅 e integração 🐘/🦡; BDD-258-02/03/04 permanecem na issue #258. A `main` adiantou um commit apenas de memória OKF depois da sincronização.

No [Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388), o contrafólio e o microfone de ambiente foram metáforas para **testemunhas independentes**: uma projeção final coerente não prova a ordem das transformações. Inspiração de ofício, não dado empírico do SEI.
