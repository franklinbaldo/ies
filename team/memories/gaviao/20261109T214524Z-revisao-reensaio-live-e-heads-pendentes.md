---
type: AgentMemory
title: "Reensaio PDF UnB com saída 0 e fila dos heads preservada"
description: "Parecer #606 no head reconciliado c7cc743 com CI 7/7; #602 e #605 já revisadas no mesmo SHA; #526 continua Draft e muito defasada."
agent: gaviao
timestamp: "2026-11-09T21:45:24Z"
tags: [revisao, bdd, ci, pdf, live, cassettes, sqlite, cafezinho]
---

# Retomar na próxima rodada

**Fato verificado:** nova revisão publicada na [#606](https://github.com/franklinbaldo/ies/pull/606#issuecomment-6752334129), head `c7cc743`. [CI oficial #42705331117](https://github.com/franklinbaldo/ies/actions/runs/42705331117) **7/7 PASS**, [rápido #42705320633](https://github.com/franklinbaldo/ies/actions/runs/42705320633) verde e `main...head` behind=0. **PRONTA PARA INTEGRAR como relatório**. `docs/validacao-live/2026-11-09-documento-incluir-externo-unb.md:84-121` preserva a primeira saída 3 indevida e relata segundo ensaio autorizado com saída 0, documento `0012273`, releitura de SHA-256 idêntico e resíduos `0012272` e `0012273` (16/25 requisições). A prova live consta do relatório; não foi reproduzida pelo Gavião. Nenhuma escrita SEI nesta revisão. O `--confirmar` público segue fechado.

**Fila sem novos heads:** [#605](https://github.com/franklinbaldo/ies/pull/605#issuecomment-6752232998) segue `ef720cd`, já revisada e com CI 7/7; documenta atribuição com POST único inconclusivo (saída 3) e possível resíduo, sem remoção ou retry. [#602](https://github.com/franklinbaldo/ies/pull/602#issuecomment-6752233968) segue `e6a9c73` 7/7 com **behind=21**, precisa de merge verdadeiro da main e novo CI. Seu BDD-259-01 CLI↔anti-IES↔SQLite é PASS sintético, cassette semântico e live NOT_RUN. [#526](https://github.com/franklinbaldo/ies/pull/526) segue Draft `5188493`, conflito e **behind=911**: preservar E2E existente de duas CLIs/POST único e escolher migração depois da última versão efetivamente integrada; não se antecipar a v27 da #602.

**Cafezinho:** visitei a [issue #388](https://github.com/franklinbaldo/ies/issues/388) e li a conversa recente sobre estado observado e autoria do efeito. A diferença entre o resultado do primeiro e segundo ensaio do PDF ilustra a importância de não reescrever historicamente evidências anteriores. Essa conexão é interpretação, não uma nova prova experimental.

**Próximo:** buscar heads novos sem repetir parecer no mesmo SHA; separar compatibilidade sintética, cassette semântico e live; exigir CI 7/7 no SHA exato e conferir sincronização. Não houve merge, implementação nem acesso ao SEI pelo Gavião.
