---
type: AgentMemory
title: "BDD-61-03: busca vazia explícita, não silêncio de um SQLite incompleto"
description: "PR #489 tem CI completo 7/7 para a CLI real contra anti-IES; cassettes GO/UnB conferidas por hash, sem paridade semântica."
agent: raposa
timestamp: "2026-11-08T17:32:33Z"
tags: [ux, bdd, anti-ies, sqlite, cassette, ci]
---

# Retomar

**Fato:** a [PR #489](https://github.com/franklinbaldo/ies/pull/489) implementa BDD-61-03 (issue [#61](https://github.com/franklinbaldo/ies/issues/61)), com busca documental sem resultado explícito por origem/cenário/unidade/termo, simulador HTML, SQLite efêmero v14 e CLI real em TLS local. [CI completo #42505193401](https://github.com/franklinbaldo/ies/actions/runs/42505193401): 7/7 PASS; [CI rápido #42505185423](https://github.com/franklinbaldo/ies/actions/runs/42505185423): PASS. A PR está Ready, sem merge pela Raposa. O RED [#42504050477](https://github.com/franklinbaldo/ies/actions/runs/42504050477) falhou por tabela ausente antes da implementação.

**Observação vs hipótese:** [inspeção no Actions #42504191547](https://github.com/franklinbaldo/ies/actions/runs/42504191547) confirmou hashes de dois ZIPs, 18 interações de GO/UnB, inclusive GET/POST `protocolo_pesquisar`. Não houve comparação dos corpos HTML, REA nem execução live. O vazio foi simulado como **hipótese explícita**; ausência de registro para outro termo não é prova de inexistência e devolve HTTP 503.

**Dependência de integração:** migração v12 reservada à #486 (PDF), v13 à #487 (cancelamento), esta fatia usa v14. Na integração, preservar a sequência das três em `banco_cassettes::ESQUEMA`; verificar conflitos/CI no SHA atualizado. Depois do merge, reavaliar os BDD-61-01/02 pendentes.

**Inspiração do [Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6724676126):** zero linhas e resposta “zero resultados” são histórias distintas com a mesma aparência. É a própria questão de identificabilidade: precisamos observar a resposta negativa, não deduzi-la do silêncio.
