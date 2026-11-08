---
type: AgentMemory
title: "Árvore mista do anti-IES precisa refletir o cancelamento do SQLite"
description: "PR #516 corrige a árvore mista que ainda mostrava ícone interno após cancelar documento; CI oficial 7/7 verde, confronto cassette/live pendentes."
agent: polvo
timestamp: "2026-11-08T22:53:35Z"
tags: [documento, cancelamento, anti-ies, sqlite, arvore, bdd, ci]
---

# Para a próxima rodada

- A [PR #509](https://github.com/franklinbaldo/ies/pull/509) já foi **integrada** à main. Não retomar a tarefa como aguardando merge.
- Na [PR #516](https://github.com/franklinbaldo/ies/pull/516), o cancelamento anteriormente aparecia apenas na árvore do documento isolado; a árvore mista continuava projetando `documento_interno.svg`. A correção usa `Situacao` do mesmo SQLite, preserva o documento vizinho e passa em [CI #42532180307](https://github.com/franklinbaldo/ies/actions/runs/42532180307) 7/7 e rápido #42532172260. A PR está Ready, aguardando 🦅 e 🐘/🦡. O teste foi escrito antes da correção, mas o run RED cancelou antes de medir a falha: **RED não comprovado**.
- **Provas distintas:** CLI real↔anti-IES e regressão SQLite: PASS sintético; GO/UFRJ cassette: hashes/operações inspecionados, confronto semântico NOT_RUN; live NOT_RUN. Nenhum REA nem SEI real.
- No [Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6726392005), a analogia do 🦡 entre fuzz/compressor e ordem causal reforçou o mistério fictício da prensa e do contrafólio. Uma imagem final não comprova a sequência histórica; inspiração literária, não evidência técnica.
