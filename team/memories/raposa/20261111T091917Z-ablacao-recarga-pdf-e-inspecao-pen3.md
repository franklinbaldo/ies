---
type: AgentMemory
title: "Recarga do tipo externo: consulta isolada e PEN3 conferida"
description: "PR #770 elimina too_many_lines sem alterar a prévia; CI 7/7, inspeção PEN3 com hashes válidos e confronto semântico bilateral pendente."
agent: raposa
timestamp: "2026-11-11T09:19:17Z"
tags: [ux, ablacao, pdf, recarga, clippy, cassette-first, ci, cafezinho]
---

# Para a próxima rodada

**Fato:** [PR #770](https://github.com/franklinbaldo/ies/pull/770), head `531e5838`, separa o POST de consulta `documento_receber` em `recarregar_tipo`, removendo uma supressão `too_many_lines` sem mudar a CLI ou o anti-IES. Diff somente de `src/documento_externo_incluir/fluxo.rs` (+27/−17). A revisão 🦅 e a integração 🐘/🦡 ainda dependem da análise independente. A [#721](https://github.com/franklinbaldo/ies/issues/721) segue aberta.

**Prova:** [CI oficial #42993698336](https://github.com/franklinbaldo/ies/actions/runs/42993698336) **7/7 PASS** e [rápido #42993688095](https://github.com/franklinbaldo/ies/actions/runs/42993688095) PASS após merge verdadeiro com a `main`, com E2E sintético já existente da CLI Rust ↔ anti-IES/SQLite. O [inspecionar-cassettes #42993258030](https://github.com/franklinbaldo/ies/actions/runs/42993258030) validou ZIP SHA-1 e JSON SHA-256 da PEN3 (9 interações); confronto **semântico bilateral NOT_RUN**, treinamento/live **NOT_RUN**. Arquivo temporário de inspeção eliminado do diff final.

**Cafezinho:** [conversa na #388](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6791860732) reforçou que POST de recarga é consulta, não cadastro. Retomar a revisão do SHA e a fila de supressões sem novo Gherkin ou subcódigo.
