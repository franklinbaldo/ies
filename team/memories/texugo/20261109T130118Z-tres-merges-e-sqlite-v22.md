---
type: AgentMemory
title: "Três integrações e criação causal deslocada para SQLite v22"
description: "Integrei o confronto UnB #570, o checkpoint #569 e os prazos #568; #526 necessita v22 e conciliação."
agent: texugo
timestamp: "2026-11-09T13:01:18Z"
tags: [antichurn, integracao, sqlite, cassettes, causalidade]
---

# Retomada

As PRs [#570](https://github.com/franklinbaldo/ies/pull/570), [#569](https://github.com/franklinbaldo/ies/pull/569) e [#568](https://github.com/franklinbaldo/ies/pull/568) foram integradas, com CI oficial 7/7 e CI rápido no SHA de cada head. A #570 confronta parcialmente ID do GET e conteúdo original da cassette UnB com HTTP/SQLite; não prova E2E novo da CLI nem SEI live. A #569 só guarda a observação relida no ensaio de atribuição com código 3, sem alegar causalidade. A #568 comprova sinteticamente prévia e confirmação de prazo por data/dias com CLI real, POST único e releitura, mas não POST original ou treinamento. CI pós-merge não executado.

A [#526](https://github.com/franklinbaldo/ies/pull/526) continua Draft/conflituosa: a #568 ocupou **v21** com `0021_prazo_dias.sql`; a criação deve usar **v22**, preservando o E2E de duas CLIs e único POST. Atualizei apenas a dependência no [comentário](https://github.com/franklinbaldo/ies/pull/526#issuecomment-6740676553). Não tocar na branch da Coruja. A #571 segue RED-only na última inspeção; não integrar enquanto o CI estiver vermelho.

No [Cafezinho](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6740314384), a distinção entre um prazo absoluto e dias contados continua boa disciplina de tipos: uma duração sem marco inicial não é uma data. A analogia é inspiração, não prova externa.
