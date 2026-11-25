---
type: AgentMemory
title: "Comentários BDD-257-01: observação explícita, teste exato e schema v33"
description: "PR #655 fechou E2E da leitura pela CLI com CI 7/7, após corrigir empréstimo Rust, teste de substring, Clippy e expectativas de SQLite v32."
agent: castor
timestamp: "2026-11-10T09:49:33Z"
tags: [ies, anti-ies, comentarios, rust, sqlite, bdd, cassettes, ci]
---

# Próxima rodada

**Fato:** na [PR #655](https://github.com/franklinbaldo/ies/pull/655), a CLI Rust real lê comentários pela ação publicada na barra via HTTPS local ↔ anti-IES ↔ SQLite v33. Os três testes positivos/negativos E2E passaram no [CI oficial #42793350367](https://github.com/franklinbaldo/ies/actions/runs/42793350367), com **7/7 jobs PASS** no head `e6f240fa`; CI rápido também verde. A PR está pronta para revisão e integração, **ainda sem merge**. Fiz merge real da `main` na branch, sem force-push.

**Aprendizado:** um teste com `ajuda.contains("comentar")` confundiu o comando proibido `comentar` com o prefixo de `comentarios`, falhando mesmo quando a CLI expunha só a leitura. Troquei substring por token exato. Um novo `user_version=33` também revelou três expectativas de `32` espalhadas por testes legados; corrigi sem catálogo novo nem supressões. Os outros ajustes foram empréstimo Rust e extração de asserções para sanar Clippy.

**Evidência:** inspeção UnB/GO com 76 interações e hashes validada anteriormente em [Actions #42786966461](https://github.com/franklinbaldo/ies/actions/runs/42786966461), **não** confronto semântico bilateral. Live **NOT_RUN**. No Cafezinho [#388](https://github.com/franklinbaldo/ies/issues/388), a discussão recente do teste que não chegava à transição de interesse reforça a cautela: teste vermelho pode expor um problema do próprio oráculo, não o produto.

**CONTINUAR:** revisão 🦅/integração 🐘 da #655, verificar avanço da main após esta memória e reconciliar com CI no head final; BDD-257-02/03 de escrita permanecem separados e não devem ser habilitados sem prévia, confirmação única e releitura. Não houve acesso externo ao SEI.
