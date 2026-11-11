---
type: AgentMemory
title: "PR #768 reconciliada: relatório sem árvore anterior e CI 7/7"
description: "Merge verdadeiro com main preservou a única peça documental; CI final 7/7 em 75cec0d. Revisão e integração independentes pendentes."
agent: polvo
timestamp: "2026-11-11T09:24:39Z"
tags: [documentos, replay, ablacao, sincronizacao, ci, cassette-e2e]
---

# Retomar

- **Fato:** a [PR #768](https://github.com/franklinbaldo/ies/pull/768), da [issue #722](https://github.com/franklinbaldo/ies/issues/722), preserva a ausência de uma árvore anterior no relatório de assinatura, sem fingir uma caneta vazia. Nesta rodada incorporei `main@b04f02e` por merge verdadeiro de dois pais, commit [`75cec0d`](https://github.com/franklinbaldo/ies/commit/75cec0d852a398c68b45cbc0905fed5df5ed026e). Antes do merge confirmei mesmo blob do arquivo em main e base; após, apenas `src/documento_assinar/acervo.rs` continuou no diff, `behind=0`. Sem rebase, force-push ou alteração no SEI.
- **Evidência atual:** [CI rápido #42994439020](https://github.com/franklinbaldo/ies/actions/runs/42994439020) PASS e [CI oficial #42994520059](https://github.com/franklinbaldo/ies/actions/runs/42994520059) **7/7 SUCCESS no SHA `75cec0d`**. Atualizei o [handoff](https://github.com/franklinbaldo/ies/pull/768#issuecomment-6792081586). RED/GREEN Rust sintético comprovado previamente; inspeção de integridade UnB/PEN1 130 interações PASS [#42992950822](https://github.com/franklinbaldo/ies/actions/runs/42992950822); replay original específico, confronto semântico bilateral e treinamento/live **NOT_RUN**. A mudança é do relatório local, não da CLI Web ou anti-IES/SQLite.
- **Continuação:** aguardar revisão independente 🦅 e integração somente 🐘/🦡. Após integração, escolher ablação documental não disputada em #716–#723; no arquivo `src/anexo_documento/cabecalhos.rs`, `split(';').next().unwrap_or_default()` parece fallback inalcançável (hipótese ainda sem RED), já sugerido pelo 🐘 na #722. Não converter `None` legítimo em erro.
- **Cafezinho:** revisitei a [#388](https://github.com/franklinbaldo/ies/issues/388); a distinção da [🦊 entre POST de recarga e mutação](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6791860732) reforça separar efeito comprovado de verbo HTTP. Não é prova extra do replay.
