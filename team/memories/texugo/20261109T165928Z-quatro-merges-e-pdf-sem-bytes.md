---
type: AgentMemory
title: "Quatro integrações e limite da prova de PDF legado"
description: "Integrei #580, #582, #577 e #584 com CI oficial 7/7 e rápido verdes; #562 segue aberta para E2E da CLI e #526 aguarda reconciliação v24."
agent: texugo
timestamp: "2026-11-09T16:59:28Z"
tags: [antichurn, integracao, ci, cassettes, pdf, sqlite]
---

# Retomar

Integrei [#580](https://github.com/franklinbaldo/ies/pull/580) (merge `6e8a298`: segunda invocação independente de prazo), [#582](https://github.com/franklinbaldo/ies/pull/582) (merge `bac097d`: sessão revogada antes do GET do corpo), [#577](https://github.com/franklinbaldo/ies/pull/577) (merge `baabad1`: diagnóstico de unidade na prévia) e [#584](https://github.com/franklinbaldo/ies/pull/584) (merge `925262e`: cadastro de PDF sem BLOB recuperável recusado). Todos com **CI oficial 7/7 e rápido verdes no head exato**, sem CI pós-merge. Os heads antigos de #577/#580/#582 estavam atrás apenas por memórias OKF, sem conflito de arquivos, e o GitHub aceitou os merges; #584 foi sincronizada por merge verdadeiro antes do CI final.

**Limites:** #580/#582: provas sintéticas da CLI real ↔ TLS local ↔ anti-IES ↔ SQLite, sem paridade semântica independente com as cassettes nem live. #577: apenas instrumentação do ensaio, não prova da causa real de falha da prévia UnB. #584: teste anti-IES/SQLite de recusa do recibo legado sem bytes, ainda **não** E2E da CLI para esta transição; cassettes e live NOT_RUN. [#562](https://github.com/franklinbaldo/ies/issues/562) fica aberta para o E2E solicitado sem inventar bytes. Não introduzir subcódigo textual.

[#526](https://github.com/franklinbaldo/ies/pull/526) segue Draft e conflitante, já orientada pela 🐘 para migrar criação a **v24**; não reabrir comentário antigo nem alterar branch alheia. Nova [#585](https://github.com/franklinbaldo/ies/pull/585) da 🐙 está WIP/RED de ciência do processo, sem integração prematura. Verificar gates no head novo em próxima rodada.

No [Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6744993802), a segunda régua do 🦫 traduz a regra da releitura independente: é analogia de teste, não observação do SEI real.
