---
type: AgentMemory
title: "BDD-60-05: recusa 403 não cria nenhum arquivo local"
description: "PR #653 integrada após E2E da CLI real/TLS/SQLite e conferência integral da pasta; CI 7/7 no head validado, corrida não observada em cassette."
agent: tartaruga
timestamp: "2026-11-10T09:07:08Z"
tags: [bdd60, anti-ies, cli, tls, sqlite, versoes, cassette-first]
---

# Para retomar

**Fato:** [PR #653](https://github.com/franklinbaldo/ies/pull/653) integrada pelo [merge de dois pais `f875c380`](https://github.com/franklinbaldo/ies/commit/f875c380763ed4becb7f85d3e3f54b86f4695ca2). [CI oficial #42787468878](https://github.com/franklinbaldo/ies/actions/runs/42787468878) 7/7 no head `aeaf56b28`. O E2E com CLI Rust real e anti-IES/TLS/SQLite preserva HTTP 403 e saída 1 após revogação entre lista e GET de uma versão, com/sem `--ocultar-inacessiveis` e com `--saida DIR`. O 🦅 chamou atenção à insuficiência de verificar apenas dois nomes de foto: testamos agora a listagem **integral** da pasta, além dos bytes do rascunho. Nenhum código produtivo foi alterado.

**Lição:** uma pré-condição local pode impedir chegar ao estado que o ensaio pretende falsificar: a primeira fixture com foto v1 divergente parou antes da rede. Corrigi o dado de entrada; não chamar essa falha de RED funcional. As cassettes UnB/PEN1 registram listas e corpos, mas **não** a revogação temporal; negativo sintético, live NOT_RUN. Compartilhado no [Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6763343348).

**Próximo ciclo:** a issue #60 permanece aberta, mas este subcaso já foi integrado; começar de novo na `main`, verificar BDD pendente e lacuna concreta sem repetir o ensaio. [Handoff #60](https://github.com/franklinbaldo/ies/issues/60#issuecomment-6763397588).
