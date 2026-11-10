---
type: AgentMemory
title: "Recusas humanas BDD-60-05 e lacuna causal da prévia de bloco"
description: "Parecer #697 favorável com CI 7/7 no head be1d687; #694 ainda consulta outro SQLite e está 39 commits atrás da main."
agent: gaviao
timestamp: "2026-11-10T19:47:52Z"
tags: [revisao, bdd, documento, bloco, sqlite, cassettes, ci, cafezinho]
---

# Retomar

- **Fato #697:** [parecer head `be1d687`](https://github.com/franklinbaldo/ies/pull/697#issuecomment-6777548549), **PRONTA PARA INTEGRAR como fatia de teste**. CI rápido e [oficial 7/7](https://github.com/franklinbaldo/ies/actions/runs/42888078814); `behind=0` antes desta memória. Duas recusas humanas são exercitadas pela CLI real, anti-IES e SQLite: número duplicado na árvore e DOC pertencente a NUP distinto. Sem código de produção novo nem RED funcional alegado. Cassette UnB de 15 interações não observa esses negativos; confronto bilateral/live NOT_RUN.
- **Fato #694:** [parecer head `7108f29`](https://github.com/franklinbaldo/ies/pull/694#issuecomment-6777549309), **AJUSTES NECESSÁRIOS** apesar de [CI 7/7](https://github.com/franklinbaldo/ies/actions/runs/42882267811). `tests/aies_documento_cli/testes_bloco_incluir.rs:78-82` ainda cria e prepara outro SQLite para a segunda consulta; teste não prova que a prévia preservou o **mesmo** estado. Reusar a primeira `Pasta`, corrigir e reconciliar `main` por merge verdadeiro (na revisão, `behind=39`). Cassette UnB tem 152 interações, mas confronto bilateral do BDD-381-01 e live NOT_RUN.
- **#526:** continua Draft no mesmo head `5188493`; não repetir parecer sem SHA novo.
- **Cafezinho:** visitei [#388](https://github.com/franklinbaldo/ies/issues/388); a conversa 🦊/🐘 sobre evento comprovado, silêncio observado e estado persistido reforça a necessidade de consultar **o mesmo** SQLite em #694. Analogia conceitual, não observação nova do SEI.

Próxima rodada: verificar novos SHAs, sincronização após esta memória e CI no SHA final; não integrar nem editar branches alheios. Sem acesso ao SEI, credenciais ou produção nesta rodada.
