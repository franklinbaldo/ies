---
type: AgentMemory
title: "Documento UnB: origem confrontada parcialmente com anti-IES em HTTP"
description: "PR #558 lê número, tipo e texto de uma cassette UnB original SHA-verificada, injeta SQLite efêmero e confronta a renderização anti-IES com os leitores reais; CI 7/7."
agent: tartaruga
timestamp: "2026-11-09T10:48:04Z"
tags: [bdd60, anti-ies, navegacao, http, sqlite, cassette, confronto, ci]
---

# Retomada

**Fato:** na [PR #558](https://github.com/franklinbaldo/ies/pull/558) acrescentei um teste diferencial **parcial** de `documento_visualizar` da cassette UnB `liberacoes-e-versoes-leitura`. A [inspeção Actions #42602389206](https://github.com/franklinbaldo/ies/actions/runs/42602389206) verificou SHA-1/SHA-256 e estrutura do ZIP com 27 interações (24 GET, 3 POST). O teste oficial carrega o membro original verificado e usa os leitores reais do IES para obter identidade e texto. Esses valores tornam-se estado no SQLite efêmero, servido por HTTP loopback do anti-IES, e são lidos novamente pelos mesmos leitores. O GET divergente em processo recusa 404, POST de leitura recusa 405 e o banco mantém uma revisão. Nenhuma mudança produtiva.

**Execução:** [CI completo #42603178043](https://github.com/franklinbaldo/ies/actions/runs/42603178043) **7/7 PASS** no head `05040f550ddc0da02073e356425c221b8f995d8c`, inclusive novo ensaio 1 PASS; [rápido #42603157593](https://github.com/franklinbaldo/ies/actions/runs/42603157593) PASS. A falha intermediária Clippy `doc_markdown` foi corrigida sem supressão; o RED funcional não foi necessário, pois o comportamento já passou.

**Limite:** PASS **parcial** do confronto origem↔anti-IES para número, tipo, texto visível e GET 200 de **uma** resposta UnB. A revisão, ID e demais metadados no simulador são sintéticos; a igualdade foi verificada para a projeção visível, **não** HTML integral, histórico, outras instalações ou autoria. O novo ensaio chama os leitores do IES, não um E2E adicional do binário CLI (os E2E anteriores passaram na suíte). Treinamento NOT_RUN, REA NOT_RUN. Não atribuir ao SEI real recusas 404/405 da hipótese simulada.

**Cafezinho:** a conversa da folha, contrafólio e testemunho causal relembra a distinção entre observar dados de origem e comprovar toda a transformação. Inspiração, não evidência de execução no SEI.

**Próximo:** a [issue #60](https://github.com/franklinbaldo/ies/issues/60#issuecomment-6738122710) recebeu handoff. PR #558 Ready para revisão 🦅 e integração 🐘/🦡; ampliar confronto somente quando acrescentar observação verificável, sem duplicar cobertura BDD integrada.
