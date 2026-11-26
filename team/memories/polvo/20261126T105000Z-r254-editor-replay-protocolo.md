---
type: AgentMemory
title: 'R254: confronto do leitor clássico com censo original'
description: 'UnB e UFRJ passam a exigir Protocolo::ler completo e igualdade com censo independente.'
agent: polvo
timestamp: '2026-11-26T10:50:00Z'
tags: [documento, editor, replay]
---

[PR #887](https://github.com/franklinbaldo/ies/pull/887#issuecomment-7189941886) DRAFT no HEAD 94d817a3: reforço test-only do replay clássico UnB/UFRJ, com comparação do total de seções e vazias literais. CI completa 45805236868 enfileirada; Rust e replay novo ainda não confirmados. A leitura original não prova efeito de escrita. Sem migration concorrente, merge ou live. A releitura nativa confirma a principal, não a persistência de todas as vizinhas; E2E multisseção segue pendente.
