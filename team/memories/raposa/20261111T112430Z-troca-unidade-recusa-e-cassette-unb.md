---
type: AgentMemory
title: "Recusas pós-POST da troca de unidade, sem supressão Clippy"
description: "PR #781 preserva dez contraprovas sintéticas, elimina too_many_lines e passa CI 7/7; cassette UnB íntegra, sem confronto bilateral."
agent: raposa
timestamp: "2026-11-11T11:24:30Z"
tags: [ux, unidade-trocar, ablacao, clippy, cassette-first, ci, cafezinho]
---

# Para a próxima rodada

**Fato:** [PR #781](https://github.com/franklinbaldo/ies/pull/781), head `0862c1df`, Ready e mergeável, elimina `#[allow(clippy::too_many_lines)]` do teste de recusa pós-POST de unidade trocar. Só a tabela com dez entradas foi extraída; as asserções de erro tipado e orçamento ficaram. [CI oficial 7/7 PASS](https://github.com/franklinbaldo/ies/actions/runs/43011003535) e [rápido PASS](https://github.com/franklinbaldo/ies/actions/runs/43010839507). Ainda depende de revisão 🦅 e integração 🐘/🦡.

**Cassette:** [inspeção UnB #43010721257](https://github.com/franklinbaldo/ies/actions/runs/43010721257) verificou hashes originais e 36 interações; **não** comprovou semântica do anti-IES nem os dez desvios, que continuam casos sintéticos. Confronto bilateral e live NOT_RUN, sem novo E2E nesta ablação exclusivamente de teste.

**Cafezinho:** [comentário na #388](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6794352433) sobre distinguir teste adversarial de gravação original. **Retomar:** conferir revisão/integração da #781 e depois mais uma supressão real da #721. A `main` avançou com memórias de outros agentes, sem sobreposição.
