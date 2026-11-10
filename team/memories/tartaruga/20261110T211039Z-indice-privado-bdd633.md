---
type: AgentMemory
title: "BDD-633: fixture do índice privado impedia testar link guardado"
description: "No E2E da modal, o diretório temporário sem 0700 desligava o índice local; ajuste explicita a colheita do NUP antes da segunda invocação. CI 7/7 no SHA 15fde980, confronto da modal com cassette e live ainda não feitos."
agent: tartaruga
timestamp: "2026-11-10T21:10:39Z"
tags: [bdd633, anti-ies, navegacao, indice-local, tls-loopback, cassette-first]
---

# Para retomar

**Fato:** na [PR #698](https://github.com/franklinbaldo/ies/pull/698), a revisão [🦅](https://github.com/franklinbaldo/ies/pull/698#issuecomment-6778958944) achou dois limites de linhas Clippy e uma segunda leitura inesperada do Controle no teste BDD-633-01 por link guardado. A fixture `Pasta::nova()` criava o diretório com permissão incompatível com a política 0700 do índice `ies.sqlite3`, desativando a colheita silenciosamente. Ajustei a pasta para 0700, exigi que o primeiro comando tivesse persistido NUP→id no índice e extraí helpers de cena/comando em vez de suprimir Clippy. [CI completo #42901452062](https://github.com/franklinbaldo/ies/actions/runs/42901452062) passou 7/7 no SHA `15fde980`; CI rápido #42901414003 também passou. A PR ainda precisa de revisão/integração e, se a base avançar após esta memória, merge verdadeiro e gates do novo head.

**Evidências:** CLI Rust real ↔ TLS local ↔ anti-IES ↔ SQLite = PASS **sintético** nos casos de sucesso, recusa e 302 com releitura. Três cards GO (48 interações) inspecionados anteriormente têm hashes íntegros, mas **não** contêm a modal ou POST; confrontação bilateral específica **NOT_RUN**, live **NOT_RUN**. Não transportar senha real, não fazer escrita externa, não enfraquecer TLS.

**Cafezinho:** li a [#388](https://github.com/franklinbaldo/ies/issues/388) e compartilhei a distinção entre um GET adicional por índice realmente desativado e uma suposta falha do caminho cacheado [aqui](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6779353781).