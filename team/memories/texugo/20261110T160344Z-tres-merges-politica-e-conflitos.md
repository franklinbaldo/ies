---
type: AgentMemory
title: "Três integrações e precedência da decisão do proprietário sobre BDD desatualizado"
description: "Integrei #672, #683 e #678 com CI 7/7 nos heads; #681 superou a saída 3 da #348, e conflitos reais de #680/#684 receberam instruções mínimas."
agent: texugo
timestamp: "2026-11-10T16:03:44.095Z"
tags: [antichurn, integracao, ci, cassettes, bdd, erros-tipados, conflitos]
---

# Retomar

**Fatos:** reli AGENTS, ficha, guia cassette-first e cinco memórias; visitei o [Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388). Integrei [#672](https://github.com/franklinbaldo/ies/pull/672) por `25658c6` (auditoria documental de bloco, CI [7/7](https://github.com/franklinbaldo/ies/actions/runs/42848876369), revisão 🦅 LGTM), [#683](https://github.com/franklinbaldo/ies/pull/683) por `c1c437a` (replay PEN1–PEN4, CI [7/7](https://github.com/franklinbaldo/ies/actions/runs/42848208667), arquivos independentes dos avanços da main) e [#678](https://github.com/franklinbaldo/ies/pull/678) por `6d5a3fa` (retirada de acompanhamento, CI [7/7](https://github.com/franklinbaldo/ies/actions/runs/42848998633), segundo veto 🦅 da saída humana corrigido com teste). Nos três, CI rápido também PASS no head exato; não rodei CI pós-merge.

**Arbitragem documentada:** [#681](https://github.com/franklinbaldo/ies/issues/681), decisão expressa do dono já integrada na main, substituiu saída 3 por saída **1 / nao_confirmado** quando releitura não comprova escrita. A #348 ainda diz 3. Não retroceder a #678 para política revogada. Registrei o fundamento na [PR](https://github.com/franklinbaldo/ies/pull/678#issuecomment-6772266717) e [handoff na #348](https://github.com/franklinbaldo/ies/issues/348#issuecomment-6772317556) para 🐘 corrigir apenas a cláusula histórica do BDD.

**Próxima esteira:** [#680](https://github.com/franklinbaldo/ies/pull/680) tem E2E de quatro CLIs válido, mas a main substituiu no teste pai `Some(3)/efeito_incerto` por `Some(1)/nao_confirmado`; [orientação mínima](https://github.com/franklinbaldo/ies/pull/680#issuecomment-6772301914) para merge verdadeiro preservando ambos os testes. [#684](https://github.com/franklinbaldo/ies/pull/684) já tem CI verde, mas sua refatoração de 46 arquivos colide com #678 em `src/processo_acompanhar.rs`, e revisão independente do novo head cabe antes de integrar; [orientação](https://github.com/franklinbaldo/ies/pull/684#issuecomment-6772302778). [#526](https://github.com/franklinbaldo/ies/pull/526) continua Draft antigo: preservar E2E com segunda CLI, reconciliar migrações apenas na retomada do dono.

**Limites de prova:** #672 é auditoria, sem novo teste; #683 tem PASS **parcial de cassettes** para seleção/tela/árvore, não autor/corpo; #678 tem PASS **sintético** CLI real ↔ anti-IES ↔ SQLite, sem confronto semântico bilateral novo. Live permanece NOT_RUN. Nenhuma requisição ao SEI, escrita em produção, subcódigo ou gate novo. Não confundir integridade de ZIP com paridade ou ausência de live com bloqueio.
