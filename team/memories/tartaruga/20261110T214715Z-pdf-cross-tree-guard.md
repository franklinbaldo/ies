---
type: AgentMemory
title: "BDD-60-05: PDF de outro processo conhecido não é observação insuficiente"
description: "RED funcional HTTP 503 vs 404 na árvore mista, correção por consulta ao dono no mesmo SQLite e GREEN 7/7 no SHA 9e6ee100; inspeção UnB 75 HTTP, negativo não capturado."
agent: tartaruga
timestamp: "2026-11-10T21:47:15Z"
tags: [anti-ies, bdd60, identidade-documental, sqlite, transporte-http, cassette-first, red-green]
---

# Para retomar

**Fato:** [PR #706](https://github.com/franklinbaldo/ies/pull/706), issue #496 usando BDD-60-05 existente. Um PDF com id público conhecido no processo B e solicitado na árvore de A devolvia 503 porque o leitor de versões não conhecia PDFs externos. [RED no CI #42906888172](https://github.com/franklinbaldo/ies/actions/runs/42906888172), head `b1b59384`: 4 testes PASS e o novo teste FAIL com HTTP 503 em lugar de 404. Não é RED de CLI binária, mas de socket HTTP real com SQLite.

**Correção:** `iesei::pdf_externo::processo_do_documento_publico` consulta o dono somente na instalação/cenário e exige ID público canônico. A árvore mista só devolve HTTP 404 quando o mesmo banco comprova dono diferente. PDF não registrado permanece 503; seleção própria 200; POST 405; GET após recusa igual; duas linhas persistidas no SQLite. Código restrito a três arquivos. [CI completo #42907610815](https://github.com/franklinbaldo/ies/actions/runs/42907610815) **7/7 PASS** e [rápido #42907383998](https://github.com/franklinbaldo/ies/actions/runs/42907383998) PASS no SHA `9e6ee1005b83f101952aa6259b8ae3ae88162f59`.

**Cassette-first:** [inspeção oficial #42906730834](https://github.com/franklinbaldo/ies/actions/runs/42906730834) conferiu SHA-1/SHA-256 dos cards `documento-mover-cancelar` (60 HTTP) e `documento-numero-e-busca` (15 HTTP) UnB. O negativo entre processos NÃO está gravado. Manifesto temporário removido. HTTP↔SQLite = PASS **sintético**; comparação específica com cassette = NOT_RUN; CLI binária↔TLS local nesta fatia = NOT_RUN (somente E2Es preexistentes do CI); live/REA = NOT_RUN.

**CONTINUAR:** após esta memória, sincronizar PR #706 com nova `main` usando merge de dois pais e revalidar gates do novo head; revisão 🦅 e integração 🐘/🦡, não fechar #496 nem #60. Nenhum SEI real, credencial, retry, subcódigo novo ou redução de TLS. No [Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6780042782), distingui dado ausente de identidade positivamente incompatível.
