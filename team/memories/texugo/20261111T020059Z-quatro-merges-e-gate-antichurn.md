---
type: AgentMemory
title: "Quatro integrações verdes e um gate de grep contestado"
description: "Integrei #734, #736, #724 e #737 com CI 7/7/rápido e parecer 🦅; fechei #726 duplicada; #739 exige revisão da nova trava e exceções de plataforma."
agent: texugo
timestamp: "2026-11-11T02:00:59Z"
tags: [antichurn, integracao, ci, bdd, cassette-first, sqlite, ablacao]
---

# Retomar

**Fatos:** li integralmente AGENTS.md, a ficha, o ciclo de cassettes e as cinco memórias recentes; visitei o [Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388) e li os comentários mais recentes. **Quatro merges confirmados** na main, sempre no head com **CI oficial 7/7 e CI rápido PASS**, revisão 🦅 e verificação de arquivos sem sobreposição material; não repeti CI por defasagem cerimonial:
- [#734](https://github.com/franklinbaldo/ies/pull/734) → [`c0c34e8`](https://github.com/franklinbaldo/ies/commit/c0c34e83385b3cdeb28e672d44389a96a675e5d7): PDF de A não passa a pertencer a B após anexação, E2E CLI/SQLite sintético.
- [#736](https://github.com/franklinbaldo/ies/pull/736) → [`22aa82d`](https://github.com/franklinbaldo/ies/commit/22aa82dbafa11338cb7a9cd16bb26c248c708f12): BDD-565-03 POST único para colunas, releitura e segunda CLI no SQLite.
- [#724](https://github.com/franklinbaldo/ies/pull/724) → [`90022f5`](https://github.com/franklinbaldo/ies/commit/90022f5e71b43999930d012e1d9f08bf79399d0b): leitura parcial de PASTA/AGUARDE recusa ausência não comprovada, E2E CLI/SQLite sintético.
- [#737](https://github.com/franklinbaldo/ies/pull/737) → [`ffaf611`](https://github.com/franklinbaldo/ies/commit/ffaf6117e358a4369fc6249450bcdf0ac97ea904): BDD-381-04 prévia de processo em bloco interno sem POST mutador, segunda CLI no mesmo SQLite; reunião ainda sem E2E específico.

Atualizei [#565](https://github.com/franklinbaldo/ies/issues/565), [#496](https://github.com/franklinbaldo/ies/issues/496), [#60](https://github.com/franklinbaldo/ies/issues/60) e [#381](https://github.com/franklinbaldo/ies/issues/381) sem encerrar cenários parciais. Fechei [#726](https://github.com/franklinbaldo/ies/pull/726), RED obsoleto sobre os mesmos quatro arquivos do GREEN já integrado na #736, com comentário causal.

**Risco pendente:** [#739](https://github.com/franklinbaldo/ies/pull/739) tem CI oficial e rápido verdes no head `5ed0784`, mas novo hook `sem-dead-code` em pre-commit/pre-push proíbe até `cfg_attr(not(unix), allow(dead_code))`, explicitamente excetuado na [#721](https://github.com/franklinbaldo/ies/issues/721); variantes de erro Unix mantidas podem receber `dead_code` fora do Linux, CI atual não prova portabilidade. [Registrei revisão antichurn](https://github.com/franklinbaldo/ies/pull/739#issuecomment-6784710864), sem mesclar. Depois dos quatro merges, `main` também alterou três arquivos de testes coincidentes com o diff de #739; exigir reconciliação funcional/CI do novo head, não apenas reaproveitar o carimbo antigo. [#526](https://github.com/franklinbaldo/ies/pull/526) segue Draft/conflitante, preservar E2E causal.

**Três níveis:** PASS sintético da CLI real ↔ anti-IES ↔ SQLite nos quatro heads. Cards UnB/GO/PEN1 lidos, hashes previamente inspecionados nos Actions; os negativos/pós-escrita correspondentes não têm confronto bilateral semântico, **NOT_RUN**. Treinamento/live **NOT_RUN**; 0 acesso ao SEI, credenciais, produção, subcódigos ou gates adicionados por esta rotina. Não houve CI pós-merge. Cafezinho: diferenciar POST de pesquisa de POST que muta é distinção semântica, não mera contagem de verbos HTTP.
