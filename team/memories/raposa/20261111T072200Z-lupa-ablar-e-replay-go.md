---
type: AgentMemory
title: "Lupa GO: separar leitura das linhas sem esconder regras"
description: "PR #763 remove um too_many_lines por extração de ler_unidades; CI oficial 7/7 e replay GO com hashes verificados, sem atribuir paridade independente ao anti-IES."
agent: raposa
timestamp: "2026-11-11T07:22:00Z"
tags: [lupa, ux, ablacao, clippy, cassette-first, go, ci, cafezinho]
---

# Retomar

**Fato:** [PR #763](https://github.com/franklinbaldo/ies/pull/763), SHA `ec3692b9`, está Ready e mergeável com CI rápido PASS e [oficial 7/7 PASS](https://github.com/franklinbaldo/ies/actions/runs/42978224269). Retirou uma supressão `too_many_lines` de `unidade_lupa::ler_pagina` por extração exata de `ler_unidades`, sem mudança pública intencional. O replay obrigatório da cassette original GO passou e verificou hashes; o confronto **independente** do anti-IES com essa resposta e live continuam NOT_RUN. A issue [#721](https://github.com/franklinbaldo/ies/issues/721) permanece aberta. 

**Cafezinho:** [comentário da Raposa](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6789641978) sobre separar responsabilidades e separar provas. **Próximo passo:** revisão 🦅 e integração 🐘/🦡, conferindo sincronização; depois escolher próxima supressão real, sem Gherkin nem novas estruturas de erro.
