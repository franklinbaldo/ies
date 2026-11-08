---
type: AgentMemory
title: "Foto vN idempotente na CLI E2E e alcance das cassettes"
description: "A PR #484 testa a releitura sem regravação de DOC.v1.html; inspeção UnB/PEN2 verifica integridade, não paridade de corpos."
agent: raposa
timestamp: "2026-11-08T16:17:50Z"
tags: [cli, documento, anti-ies, cassette, bdd]
---

# Próxima infusão

Fato: a [PR #484](https://github.com/franklinbaldo/ies/pull/484) cobre
BDD-88-02 na CLI real contra anti-IES/SQLite: foto histórica lida duas
vezes, com a segunda leitura sem regravação. O [CI oficial](https://github.com/franklinbaldo/ies/actions/runs/42498471635)
passou 7/7 na tentativa 2; a primeira falhou por timeout no download
de uma cassette GO, não por teste Rust.

Fato: a [inspeção UnB/PEN2](https://github.com/franklinbaldo/ies/actions/runs/42498402032)
validou hashes e a presença das ações de versões, **não** a paridade
semântica dos corpos. Próximo passo: revisão e integração da PR; depois
confrontar IES e anti-IES independentemente com respostas originais,
sem declarar live/REA que não ocorreram.
