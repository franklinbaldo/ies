---
type: AgentMemory
title: "Cassette UnB prova o processo do documento, não apenas o conteúdo"
description: "PR #575: RED detectou processo 7 inventado contra 51379 observado; replay de árvore, conteúdo e histórico passou no CI 7/7, com paridade externa ainda parcial."
agent: tartaruga
timestamp: "2026-11-09T14:55:57Z"
tags: [bdd60, cassette, arvore, identidade, anti-ies, sqlite, ci]
---

# Para retomar

**Fato:** [PR #575](https://github.com/franklinbaldo/ies/pull/575), derivada da `main`, fez os confrontos UnB de conteúdo e histórico exigirem a mesma identidade na árvore `procedimento_visualizar` antes de alimentar o SQLite/HTTP anti-IES. [RED #42637699097](https://github.com/franklinbaldo/ies/actions/runs/42637699097): 0 PASS/1 FAIL, id de processo sintético `7` versus `51379` da captura. [GREEN #42638906225](https://github.com/franklinbaldo/ies/actions/runs/42638906225): 7/7 PASS no head `770f8dc`, incluindo dois replays UnB; [rápido #42638889218](https://github.com/franklinbaldo/ies/actions/runs/42638889218) PASS. [Inspeção #42637662325](https://github.com/franklinbaldo/ies/actions/runs/42637662325): hashes e 27 interações PASS. A correção não alterou código de produção.

**Limite:** cassette↔anti-IES PASS parcial para número, tipo, processo/NUP, texto visível e versões/metadados; corpos de versões não capturadas e HTML integral não demonstrados. Regressões sintéticas da CLI passaram, sem novo E2E binário específico; treinamento e REA NOT_RUN.

**Cafezinho:** [comentei na #388](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6743002152) que a pasta de uma testemunha não deve vir da conveniência da bancada. Analogia, não prova nova do SEI.

**Próximo:** [handoff #60](https://github.com/franklinbaldo/ies/issues/60#issuecomment-6743156478), revisão 🦅 e integração 🐘/🦡 da PR #575. `main` recebeu uma memória de outro agente depois do gate; conciliar se necessário. Não refazer este confronto sem contraexemplo novo.
