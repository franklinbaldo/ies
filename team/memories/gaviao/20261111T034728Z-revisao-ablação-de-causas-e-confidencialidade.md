---
type: AgentMemory
title: "Correção de buffers UTF-8 privados e revisões de causas em quatro PRs"
description: "Pareceres #752/#750/#747/#746 publicados: a #752 remove buffers privados de erros UTF-8, CI oficial ainda com prek pendente; demais PRs 7/7 no head antigo e atrás da main."
agent: gaviao
timestamp: "2026-11-11T03:47:28Z"
tags: [revisao, rust, privacidade, causas, antipatterns, ci, cassettes, cafezinho]
---

# Retomar na próxima rodada

- **#752** [parecer no head `3f765ee`](https://github.com/franklinbaldo/ies/pull/752#issuecomment-6786421701): correção direta do risco da #745; nas três fronteiras UTF-8, `Utf8Error` preserva causa tipada sem manter o buffer. Testes de `Debug`, `Display` e `source()` com sentinela sintética. [CI rápido](https://github.com/franklinbaldo/ies/actions/runs/42954628135) PASS; [oficial](https://github.com/franklinbaldo/ies/actions/runs/42954689785) 6/7 no último exame, `prek` ainda em execução. Branch `behind=0` antes desta memória; conferir novamente ao final. Descrição da PR ainda dizia "RED em preparação"; não atribuir RED não executado.
- **#750** [parecer](https://github.com/franklinbaldo/ies/pull/750#issuecomment-6786422201), head `f05048c`: ablação das duas supressões via `TelaServida` existente, sem novo wrapper. CI oficial [7/7](https://github.com/franklinbaldo/ies/actions/runs/42952460345). Comparação `behind=1`; merge verdadeiro e CI novo antes de integrar.
- **#747** [parecer](https://github.com/franklinbaldo/ies/pull/747#issuecomment-6786422606), head `cae6e6e`: `#[source]` nas causas pós-POST e pós-releitura; RED observado e [GREEN 7/7](https://github.com/franklinbaldo/ies/actions/runs/42949627184). `behind=15`, branch não mesclável sem reconciliação.
- **#746** [parecer](https://github.com/franklinbaldo/ies/pull/746#issuecomment-6786423087), head `89f0b02`: preserva `ErrorKind` e `FalhaDaGravacao`; [GREEN 7/7](https://github.com/franklinbaldo/ies/actions/runs/42949283012), RED cancelado. `behind=15`; `ErrorKind` não mantém todo o `io::Error`.
- **Provas:** testes técnicos/CI e E2E preexistentes não equivalem a confrontos semânticos bilaterais com cassettes. Cards UnB de prazo (33), atribuição (24), criação documental (32/65) consultados; sem confronto novo. Live NOT_RUN. Erro local de filesystem e ocultação de buffer privado não exigem cassette SEI. Sem escrita SEI nem credenciais.
- **Cafezinho #388:** lidas as [conversas recentes](https://github.com/franklinbaldo/ies/issues/388), especialmente a 🦫 sobre `FalhaDaGravacao` no cache e a 🦊 sobre `TelaServida`. Inspiração de revisão: uma causa precisa não pode se tornar um vazamento de seu payload privado; tipagem e confidencialidade precisam andar juntas, sem criar mais tipos do que necessário.

Próximo ciclo: verificar conclusão do `prek` na #752, sincronizações com `main`, heads novos e não duplicar parecer por SHA.
