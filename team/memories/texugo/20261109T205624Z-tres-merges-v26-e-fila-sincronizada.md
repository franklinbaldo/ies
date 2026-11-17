---
type: AgentMemory
title: "Três merges, v26 efetiva e continuação sem reimplementar"
description: "Integrei #597, #603 e #601 com CI oficial 7/7 e rápido verdes; v26 é blocos, andamento #602 precisa v27; #604 aguarda sincronização e novo CI."
agent: texugo
timestamp: "2026-11-09T20:56:24Z"
tags: [antichurn, integracao, sqlite, cassettes, live, ci]
---

# Retomar

A [#597](https://github.com/franklinbaldo/ies/pull/597) entrou na `main` pelo merge `7ef044b`: **v26 é efetivamente `0026_blocos_listados.sql`**, com CLI real ↔ anti-IES ↔ SQLite **PASS sintético** no head `3fab873`, CI 7/7 e rápido. Integridade de cassettes UnB/GO observada; confronto semântico independente e live **NOT_RUN**. A [#603](https://github.com/franklinbaldo/ies/pull/603) entrou por `9069671`: somente leitura live de candidato UnB, **não** atribuição. A [#601](https://github.com/franklinbaldo/ies/pull/601) entrou por `087c829`: corrige três erros observados na inclusão de PDF externo; o documento live anterior foi criado e relido com hash igual, mas a CLI antiga saiu 3 indevidamente. Nova confirmação live com correção **NOT_RUN** e não autorizada nesta rodada. Os três heads tinham CI oficial 7/7 e rápido verde; não atribuir CI pós-merge.

Próximo: [#602](https://github.com/franklinbaldo/ies/pull/602) já tem E2E sintético da prévia de andamento, mas colide na migração 26; [orientei](https://github.com/franklinbaldo/ies/pull/602#issuecomment-6751247480) merge verdadeiro de main, **v27 se livre**, registrador/asserts e CI novo, sem recomeçar. [#604](https://github.com/franklinbaldo/ies/pull/604) passou no head `64f1f11` (CI 7/7 e rápido), mas ficou **44 commits atrás**; [orientei](https://github.com/franklinbaldo/ies/pull/604#issuecomment-6751248296) apenas sincronização e novos gates. [#526](https://github.com/franklinbaldo/ies/pull/526) permanece Draft; criação usa a primeira versão efetivamente livre na hora, provavelmente v28 se #602/v27 entrar antes.

No [Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6751015072), a analogia da película e legenda lembra que **resíduo documental realmente observado** e **código de saída 3** são fatos distintos. Ficção, não nova evidência.

Nenhuma escrita em SEI, acesso à produção, credencial, retry ou subcódigo nesta rodada.
