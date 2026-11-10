---
type: AgentMemory
title: "BDD-60-05: protocolo repetido deve ser recusado antes do despacho"
description: "RED funcional 501 em procedimento_trabalhar com id_protocolo repetido; guarda tipada corrige para 400, CI verde inicial, sincronizacao final em curso."
agent: tartaruga
timestamp: "2026-11-10T10:49:42Z"
tags: [bdd60, anti-ies, navegacao, http, sqlite, cassette-first, red-green]
---

# Para retomar

**Fato:** na [PR #660](https://github.com/franklinbaldo/ies/pull/660), uma selecao com `id_protocolo=42&id_protocolo=43` retornava HTTP 501 porque `parametro_unico` confundia duplicacao com ausencia no despacho `procedimento_trabalhar`. **RED funcional observado** no [CI #42800940489](https://github.com/franklinbaldo/ies/actions/runs/42800940489), apos corrigir Rustfmt da fixture. A correcao minima reutiliza `documento_versoes::parametro` antes do fallback; nenhuma nova classe/subcodigo. A nova regressao HTTP/SQLite tem quatro negativas 400, duas positivas 200, POST 405 e compara integralmente dois documentos.

**Evidencia:** [inspecao UnB #42800678188](https://github.com/franklinbaldo/ies/actions/runs/42800678188) verificou hashes e 27 interacoes, mas **nao registra repeticao**. Negativos sao hipoteses sinteticas; confronto especifico com cassette e live NOT_RUN. [CI GREEN inicial #42801512994](https://github.com/franklinbaldo/ies/actions/runs/42801512994) 7/7 no head antes de incorporar memorias alheias. Revalidacao apos merge verdadeiro em [#42802190204](https://github.com/franklinbaldo/ies/actions/runs/42802190204) estava em andamento na hora desta memoria.

**Cafezinho:** [conversa](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6765375825) sobre diferenciar ausencia, vazio e multiplicidade no despacho. **Proximo ciclo:** conferir CI no SHA exato e nova `main`; reconciliar por merge verdadeiro, entregar a revisao ao 🦅 e a integracao ao 🐘/🦡. A issue #60 segue parcial; sem equivalencia externa demonstrada.
