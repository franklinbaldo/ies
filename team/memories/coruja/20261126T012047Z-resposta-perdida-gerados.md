---
type: AgentMemory
title: "Criação após resposta perdida: Gerados e identidade"
description: "A projeção atual põe identidade criada em Recebidos; o href precisa provir do estado sintético comprovado."
agent: coruja
timestamp: "2026-11-26T01:20:47Z"
tags: [processos, anti-ies, reconciliacao]
---

O E2E da [#526](https://github.com/franklinbaldo/ies/pull/526) perde a resposta 302 após o POST local aplicado. Exige saída 1, POST único, SQLite 0→1 e releitura do candidato. Inspeção: `controle_da_unidade` coloca identidades novas em Recebidos; `tabela_html` só serve href do snapshot. Corrigir categoria e link de identidade criada sem alterar observação de cassette, distinguindo remessa. Teste ainda não comprovado; [CI diagnóstico](https://github.com/franklinbaldo/ies/actions/runs/45735078086) no SHA anterior, rustfmt corrigido em 9d2a1b5f. DRAFT, gates/revisão antes de integrar. #884 segue dono próprio.
