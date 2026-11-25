---
type: AgentMemory
title: "BDD-381-02: caixa oferecida é identidade, não posição"
description: "PR #701 corrigida no head 743fe596 após revisão do Gavião: nome/ID/número SEI alinhados na seleção multicaixa; CI oficial 7/7 e rápido PASS. Solicitei nova revisão; integrar antes de iniciar BDD-381-03."
agent: raposa
timestamp: "2026-11-10T21:40:58Z"
tags: [ux, bloco, bdd-381-02, anti-ies, sqlite, revisao, cassette-first]
---

# Retomar

Fato: na [PR #701](https://github.com/franklinbaldo/ies/pull/701), o bloqueio de seleção por `chkDocumentosItem0` foi corrigido no SHA `743fe596`. A prova de duas caixas usa CLI real → TLS local → anti-IES → SQLite e releitura por outra CLI; [CI oficial #42906304666](https://github.com/franklinbaldo/ies/actions/runs/42906304666) **7/7 PASS**, [rápido #42906285778](https://github.com/franklinbaldo/ies/actions/runs/42906285778) **PASS**. A atualização do handoff está na [#381](https://github.com/franklinbaldo/ies/issues/381#issuecomment-6779983576); reavaliação independente 🦅 do SHA corrigido ainda é necessária. Não iniciar BDD-381-03 antes da integração.

Limites: [cassette UnB](https://github.com/franklinbaldo/ies/blob/main/cassettes/unb/bloco-incluir-anotar-retirar-unb--c3232a87e814.md) teve hash e GET→POST 302→GET conferidos, mas não registra corpo do POST nem efeito; confronto bilateral e live **NOT_RUN**. RED comportamental da contraprova multicaixa não foi observado, GREEN sim. Zero escrita externa.

No [Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6779678907), o lembrete sobre identidade distinta de posição guiou a correção: a legenda de uma caixa permanece vinculada ao documento, não à ordem na tela.
