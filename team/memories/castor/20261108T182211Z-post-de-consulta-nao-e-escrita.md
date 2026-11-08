---
type: AgentMemory
title: "Método POST não determina efeito de escrita"
description: "BDD-258-01: o ensaio IES ↔ anti-IES usou dois POSTs de consulta e nenhum POST mutador; classificar efeitos pelo método gerou falso RED."
agent: castor
timestamp: "2026-11-08T18:22:11.681Z"
tags: [bdd, rust, sqlite, http, anti-ies, cassette]
---

# Para retomar com segurança

Na [PR #491](https://github.com/franklinbaldo/ies/pull/491), o teste do binário real de `ies processo relacionar` chegou ao servidor anti-IES com **seis requisições**, incluindo dois POSTs de consulta (`protocolo_pesquisa_rapida` e `protocolo_RI1023`). Uma asserção que classificava todo POST como mutador falhou no [CI #42508697726](https://github.com/franklinbaldo/ies/actions/runs/42508697726). Corrigiu-se o teste para distinguir **operação/efeito**, não apenas verbo HTTP. O BDD sintético de prévia passou no [CI #42509112405](https://github.com/franklinbaldo/ies/actions/runs/42509112405), embora a suíte completa anterior ainda estivesse vermelha por uma asserção legada de versão SQLite.

O modelo `iesei::relacionamentos` explicita **não observado** (`None`) versus **observado vazio** (`Some([])`) e só oferece a ação na árvore quando existe observação. O card PEN1 foi inspecionado de verdade no [Actions #42508512253](https://github.com/franklinbaldo/ies/actions/runs/42508512253) (duas cassettes, 53 interações íntegros), **não** houve confronto semântico independente ou live.

A #491 reservou a migração **v15**. Não integrar antes da sequência #487 (v13) e #489 (v14), depois fazer merge verdadeiro de `main` e CI 7/7 no head final. Não criar novos subcódigos nem promover hipótese sintética a evidência de SEI real.
