---
type: AgentMemory
title: "BDD-60-05: negativas HTTP preservam snapshot completo das versões"
description: "PR #647 adiciona 400/404/405 sintéticos derivados das cassettes e verifica imutabilidade integral no SQLite; CI 7/7; sem RED funcional ou live."
agent: tartaruga
timestamp: "2026-11-10T07:19:11Z"
tags: [bdd60, anti-ies, cassette-first, http, sqlite, identidade, navegacao]
---

# Para retomar

**Fato:** [PR #647](https://github.com/franklinbaldo/ies/pull/647) Ready, head `7352e2f0`, CI oficial [#42773817864](https://github.com/franklinbaldo/ies/actions/runs/42773817864) **7/7** e rápido [#42773771827](https://github.com/franklinbaldo/ies/actions/runs/42773771827) PASS. A mudança é apenas nos testes de confronto GO/UFRJ/PEN1–PEN4: versões inexistentes (404), identidade contraditória (400), POST de conteúdo (405) e verificação do documento **inteiro** no SQLite após todas as requisições. Uma falha intermediária Clippy `too_many_lines` foi corrigida por extração de helpers sem supressão. Branch recebeu a `main` por merge verdadeiro (behind=0 antes desta memória).

**Evidência:** as identidades e versões positivas vêm de cassettes previamente SHA-verificadas; as **novas recusas** são casos sintéticos derivados, não gravações externas. CLI real↔anti-IES↔SQLite PASS sintético **preexistente reexecutado**, sem novo E2E binário; ambos↔cassette PASS parcial preexistente reexecutado; live/REA NOT_RUN. Nenhum RED funcional, alteração produtiva, novo subcódigo ou acesso externo.

**Retomada:** [handoff na #60](https://github.com/franklinbaldo/ies/issues/60#issuecomment-6761476990); [Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6761492027). A diferença entre contagem preservada e estado integral preservado merece reaparecer em outras operações, mas não cria BDD novo. Revisão 🦅, integração 🐘/🦡; se a `main` avançar, merge verdadeiro e revalidação no novo SHA. Manter #60 aberta para lacunas de paridade externa.
