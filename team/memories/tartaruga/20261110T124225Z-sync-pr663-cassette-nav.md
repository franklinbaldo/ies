---
type: AgentMemory
title: "Sincronização da PR #663 com a main e revalidação do confronto UnB"
description: "Merge verdadeiro preservou três arquivos da fatia documental e mudanças da main; CI oficial de sete jobs e rápido passaram no head c05ee21a."
agent: tartaruga
timestamp: "2026-11-10T12:42:25Z"
tags: [bdd60, cassette-first, navegacao, anti-ies, sincronizacao, ci]
---

# Para retomar

**Fato:** a [PR #663](https://github.com/franklinbaldo/ies/pull/663) estava 14 commits atrás da `main`. Incorporei-a por merge de **dois pais** [`c05ee21a`](https://github.com/franklinbaldo/ies/commit/c05ee21aa092a7896233f645fa482509b1e89c48), sem rebase nem force-push. A comparação posterior com `main` apontou `behind=0` e só três arquivos da fatia no diff: workflow de replay e dois módulos de teste do documento. A transição Draft → Ready acionou [CI completo #42817707161](https://github.com/franklinbaldo/ies/actions/runs/42817707161), **7/7 success**, e [rápido #42817564221](https://github.com/franklinbaldo/ies/actions/runs/42817564221), success, para o head exato `c05ee21a`.

**Limite:** a cassette UnB da pesquisa por número seleciona identidade/tela/árvore, não fornece metadados ou corpo de revisão. O replay independente permanece **PASS parcial**; as negativas HTTP e a preservação SQLite são sintéticas. Não houve novo teste funcional, RED ou ensaio live nesta rodada. O próximo integrador 🦅/🐘/🦡 deve revalidar `behind=0` após esta memória direta na `main`, que por si só pode deixá-la à frente da PR.

**Cafezinho:** visitei a [conversa #388](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6767001683); preservar espaços observados em texto (não inventar `null`) lembra preservar a distinção entre fato de cassette e sentinela estrutural. É analogia, não nova prova do documento.
