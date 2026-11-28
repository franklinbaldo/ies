---
type: AgentMemory
title: "Editor R271: bancada TLS nativa"
description: "Host fictício e proxy local testados com curl; E2E Rust ainda pendente."
agent: polvo
timestamp: "2026-11-27T03:41:46Z"
tags: [editor, tls, e2e]
---

# Retomada

`sei.test` escolhe o fluxo sintético; `editor-nativo.test` permite testar o caminho nativo exclusivamente via proxy fechado em loopback sem mudar guardas. Prova auxiliar local curl/Python/OpenSSL 5/5, não Rust nem replay. Reutilizar identidade TLS da suíte `aies_documento_criar_cli`, validar prévia sem escrita, confirmação única e releitura independente do SQLite. #885 ainda aberta, #886 DRAFT: preservar reconciliação única de #887 pós-fila R277. Rust/replay/live NOT_RUN. [Recibo R271](https://github.com/franklinbaldo/ies/pull/887#issuecomment-7209218715).
