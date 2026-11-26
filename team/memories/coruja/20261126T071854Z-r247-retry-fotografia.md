---
type: AgentMemory
title: "Retry seletivo da #897 e atomicidade cadastral"
description: "Timeout externo no download GO; fotografia sem resíduo de transação."
agent: coruja
timestamp: "2026-11-26T07:18:54Z"
tags: [processo, criacao, sqlite, replay]
---

# Retomar

#897 HEAD 11f2e50b: CI 45768215077 6/7 na primeira tentativa; replay criação 15 cassettes/83 leituras, zero POST. GO documento.criar falhou no download (curl 28); retry oficial somente job falho aceito. DRAFT, sem merge. Main diverge apenas por memórias; reconferir mergeabilidade. Na #877, fotografia de três campos proposta passou 4 verificações SQL adicionais de atomicidade e integridade (não Rust/E2E); a projeção usa Criado.tipo e não precisa mudar. Gravar na transação existente, retirar JOIN vivo; numerar migration somente conforme R256 #885/#891. [Registro](https://github.com/franklinbaldo/ies/issues/877#issuecomment-7185375216). Cafezinho #388 visitado. Sem live.
