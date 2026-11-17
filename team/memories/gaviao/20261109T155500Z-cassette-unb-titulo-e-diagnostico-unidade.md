---
type: AgentMemory
title: "Recusa de título documental e diagnóstico de unidade na prévia"
description: "PRs #578 e #577: inspeção de integridade UnB passou, Clippy encontrou empréstimos redundantes e head novo corrigiu; diagnóstico da unidade é só instrumento sintético."
agent: gaviao
timestamp: "2026-11-09T15:55:00Z"
tags: [revisao, anti-ies, ci, cassettes, documento, atribuicao, cafezinho]
---

# Retomada

**Fatos:** [#578](https://github.com/franklinbaldo/ies/pull/578) testa BDD-60-05 com a CLI real em TLS local, anti-IES e SQLite: identidade de título alterada entre árvore e GET, recusa tipada sem conteúdo de outro documento. [Inspeção UnB #42647739691](https://github.com/franklinbaldo/ies/actions/runs/42647739691) confirmou hashes e 27 interações, **não** transição de título nem paridade. No head `8c2d6c2`, CI 6/7 revelou quatro `needless_borrow` em `tests/support/documento_interprocessos.rs:244/260`; [parecer](https://github.com/franklinbaldo/ies/pull/578#issuecomment-6744476070). O head posterior `32cc66a` retirou os borrows redundantes; verificar CI completo do SHA antes de aprovar integração.

[#577](https://github.com/franklinbaldo/ies/pull/577#issuecomment-6744443492) `cec619a`: CI rápido verde; relatório de prévia insere `unidade_consta_na_arvore` a partir de campos de erro já publicados, sem divulgar nomes. É **teste/instrumento**, não novo BDD nem evidência de que a diferença de unidade causou a falha observada; RED declarado pendente deve ser executado ou reclassificado. [#526](https://github.com/franklinbaldo/ies/pull/526) continua Draft conflituosa no head antigo; após merge da main, criação precisa da próxima migração livre (última observada 0022).

**Cafezinho:** visitei [#388](https://github.com/franklinbaldo/ies/issues/388) e li a [nota do Polvo](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6743461075): nome do usuário e atividade precisam aparecer na mesma linha para provar autoria. Analogia narrativa, não observação nova de SEI.

**Próximo passo:** comparar heads da fila, não duplicar parecer e distinguir PASS sintético, inspeção de cassette, confronto semântico e treinamento NOT_RUN.
