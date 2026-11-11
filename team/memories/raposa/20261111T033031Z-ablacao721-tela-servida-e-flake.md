---
type: AgentMemory
title: "Ablação #721: TelaServida unifica contexto de prazo e atribuição"
description: "PR #750 eliminou dois allows de muitos argumentos, com CI 7/7 no SHA f05048c2; o primeiro CI também reproduziu o flake SQLite #609."
agent: raposa
timestamp: "2026-11-11T03:30:31Z"
tags: [ablacao, ux, tela-servida, processo-prazo, processo-atribuir, ci, sqlite, cafezinho]
---

# Retomar

**Fato:** a [PR #750](https://github.com/franklinbaldo/ies/pull/750), head `f05048c224cd1757959f2819e799101dedb9264e`, removeu duas supressões `too_many_arguments` substituindo parâmetros soltos de URL, HTML e charset pelo `TelaServida` já existente. Quatro arquivos, +38/-45 linhas; sem tipos, subcódigos ou BDD novos. [CI rápido #42952445983](https://github.com/franklinbaldo/ies/actions/runs/42952445983) PASS e [CI oficial #42952460345](https://github.com/franklinbaldo/ies/actions/runs/42952460345) 7/7 PASS no SHA final. PR Ready, mergeável e `behind=0` antes deste commit de memória.

**Contraprovas:** no [primeiro CI #42951703248](https://github.com/franklinbaldo/ies/actions/runs/42951703248/attempts/1), Clippy detectou 102 linhas em `executar_previa` depois da inclusão da tela, corrigidas sem `allow`; independentemente, o teste de duas conexões SQLite falhou na asserção da linha 327. Registrei a recorrência na [#609](https://github.com/franklinbaldo/ies/issues/609#issuecomment-6786061555); não alterei o índice nem desativei teste. Não alegar RED comportamental.

**Três níveis:** suíte sintética existente (incluindo CLI/anti-IES) PASS no CI; cards UnB [prazo](https://github.com/franklinbaldo/ies/blob/main/cassettes/unb/processo-prazo-unb--48614e412f28.md) e [atribuição](https://github.com/franklinbaldo/ies/blob/main/cassettes/unb/processo-atribuir-unb--bf0d4327d7d9.md) consultados, confronto bilateral específico NOT_RUN; treinamento/live NOT_RUN. Não houve acesso externo ao SEI.

**Cafezinho:** [conversa #388](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6785971318) sobre reconhecer que a tela servida tem uma só origem. **CONTINUAR:** revisão independente 🦅 e integração 🐘/🦡 após conferência de sincronização; #721 continua aberta para outras supressões.
