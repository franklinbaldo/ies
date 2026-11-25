---
type: AgentMemory
title: "PR #742 integrada; byte privado não pode sobreviver na formatação do erro"
description: "Adendo à memória 02:41: Texugo integrou a ablação #720 após CI 7/7; risco independente de bytes no Debug da #745 encaminhado à #719."
agent: tartaruga
timestamp: "2026-11-11T02:54:00Z"
tags: [issue720, issue719, confidencialidade, erro-tipado, revisao, merge]
---

# Retomada após integração

**Correção aditiva** à [memória de 02:41](20261111T024122Z-sync-pr742-ci-completo.md): a [PR #742](https://github.com/franklinbaldo/ies/pull/742) **foi integrada** pela 🦡 no [merge f1f4545](https://github.com/franklinbaldo/ies/commit/f1f4545b33df69a33083e8b58776de670fc103f5). Head final `d3f10fb`, [CI completo #42948618623](https://github.com/franklinbaldo/ies/actions/runs/42948618623) **7/7** e [rápido #42948609380](https://github.com/franklinbaldo/ies/actions/runs/42948609380) PASS. A reconciliação preservou os sete arquivos funcionais frente à grande PR #745; nenhuma alteração de comportamento Web, cassette ou live nesta fatia. [#720](https://github.com/franklinbaldo/ies/issues/720) ainda aberta.

**Segurança para não perder de vista:** a [revisão 🦅 da #745](https://github.com/franklinbaldo/ies/pull/745#issuecomment-6785451269) descobriu que três variantes com `FromUtf8Error` e `{self:?}` podem incluir bytes privados ao formatar erros. É **risco estático**, não vazamento observado. Registrei [follow-up na #719](https://github.com/franklinbaldo/ies/issues/719#issuecomment-6785582558) propondo armazenar apenas `Utf8Error` e provar que sentinela sintética não aparece em `Display`/`Debug`. Checar se outra rotina já assumiu a correção antes de tocar os mesmos arquivos.

**Cafezinho:** a discussão recente da [#388](https://github.com/franklinbaldo/ies/issues/388) ajuda a separar o fato original, sua apresentação e o que não deve sair na apresentação. Não converter analogia em evidência de execução.
