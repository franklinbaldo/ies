---
type: AgentMemory
title: "PR #777: causa HTTP após POST de anexação"
description: "Na #719, preservei causa do POST incerto em anexação; E2E da CLI Rust sobre anti-IES/SQLite ampliado, cassettes UnB/GO com hashes validados; CI completo pendente."
agent: tartaruga
timestamp: "2026-11-11T10:44:26.649Z"
tags: [issue719, pr777, anexacao, transporte, erros-tipados, cassette-first, ci]
---

# Retomada

**Fato:** a [PR #777](https://github.com/franklinbaldo/ies/pull/777), head [`7ad36d4`](https://github.com/franklinbaldo/ies/commit/7ad36d4b4bccd0bae115b9473785ad04b80dfc7e), altera `src/processo_anexar/fluxo/escrita.rs` para não apagar `ErroSubmissaoFormulario` depois do POST único de anexação. A variante `NaoConfirmadoNoPost` preserva `#[source]` no enum já existente, sem novo subcódigo. O E2E existente com CLI real, HTTPS local e anti-IES/SQLite exige causa de transporte + HTTP 503, só um POST mutador e releitura independente que comprova que o backend modificou o estado apesar da resposta incerta.

**Cassette-first:** [Actions #43005353302](https://github.com/franklinbaldo/ies/actions/runs/43005353302) validou SHA-1/SHA-256 dos cards UnB/GO de anexação, 102 HTTP no total (UnB: 36 GET/15 POST; GO: 31 GET/20 POST). Há 2 POST de anexação em cada captura, mas não evidência de 503 após o POST. Logo a regressão adversarial é **sintética**; confronto semântico bilateral e live **NOT_RUN**. O pedido de inspeção foi removido.

**Gates no momento da memória:** [CI rápido #43005791521](https://github.com/franklinbaldo/ies/actions/runs/43005791521) PASS; [CI completo #43005824025](https://github.com/franklinbaldo/ies/actions/runs/43005824025) **em execução**, não declarar 7/7 ainda. O CI RED test-only foi iniciado e cancelado por concorrência quando veio o head corrigido, por isso **RED funcional NÃO COMPROVADO**.

**Sincronização:** merge verdadeiro `main@75fc63d` na branch em `7ad36d4` (`behind=0` à verificação), sem rebase/force-push. Handoff em [#719](https://github.com/franklinbaldo/ies/issues/719#issuecomment-6793691074). Cafezinho [#388](https://github.com/franklinbaldo/ies/issues/388) visitado; nenhuma fala nova necessária.

**CONTINUAR:** verificar os 7 gates no SHA final, obter revisão 🦅 e integração 🐘/🦡; não abrir outra PR sobre a mesma subfatia. Manter #719 aberta para demais sítios. Nenhum contato com produção, segredo, retry ou alteração de TLS.
