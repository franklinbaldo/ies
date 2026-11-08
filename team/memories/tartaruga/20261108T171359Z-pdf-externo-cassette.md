---
type: AgentMemory
title: "PDF externo: identidade, bytes originais e limites da observação"
description: "BDD-60-03 na PR #486 cobre CLI real, GET binário e estado SQLite v12; os cards PEN1/PEN2/PEN4 foram inspecionados por hash, sem provar paridade do PDF."
agent: tartaruga
timestamp: "2026-11-08T17:13:59Z"
tags: [documento, pdf, anti-ies, cassette, bdd]
---

# Retomar na próxima onda

[PR #486](https://github.com/franklinbaldo/ies/pull/486): BDD-60-03 exercita a CLI real sobre TLS loopback, com cadastro de PDF e releitura do original no SQLite efêmero v12. A inspeção [Actions #42503727199](https://github.com/franklinbaldo/ies/actions/runs/42503727199) confirmou hashes e três GETs registrados de `documento_download_anexo` nos cards PEN1/PEN2/PEN4; **não** comparou bytes de PDF com o anti-IES nem executou live. Houve RED [#42501360775](https://github.com/franklinbaldo/ies/actions/runs/42501360775), quando a árvore não publicava o anexo; revisar gates finais e integração antes de nova fatia. A conversa no [Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388) sobre identificabilidade lembra por que uma prova sintética não substitui uma testemunha externa.
