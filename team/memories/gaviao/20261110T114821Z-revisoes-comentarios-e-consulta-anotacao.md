---
type: AgentMemory
title: "Revisão da confirmação de comentários e do null da anotação"
description: "PR #661 com E2E causal e CI 7/7; PR #662 com erro de classificação de textarea contendo espaços, apesar de CI verde."
agent: gaviao
timestamp: "2026-11-10T11:48:21Z"
tags: [revisao, bdd, comentarios, anotacao, ausencia-observada, cassettes, ci, cafezinho]
---

# Próxima tomada

**Fato #661:** parecer [head `653068b`](https://github.com/franklinbaldo/ies/pull/661#issuecomment-6766509113); [CI oficial #42809863043](https://github.com/franklinbaldo/ies/actions/runs/42809863043) 7/7. O BDD-257-02 usa CLI real + TLS + anti-IES + SQLite, POST único, id e texto na releitura, segunda CLI e um 503 pós-POST que sai com 3 sem repetir. Na comparação anterior à memória, `behind=0`; reavaliar após avanço da main. Lista inicialmente vazia e confronto bilateral/live ainda não demonstrados. Há linha duplicada no documento de evidências, não bloqueante.

**Fato #662:** parecer [head `833ee5e`](https://github.com/franklinbaldo/ies/pull/662#issuecomment-6766509740); [CI oficial #42807803861](https://github.com/franklinbaldo/ies/actions/runs/42807803861) 7/7, mas `behind=1` na comparação. `src/processo_anotar_cli.rs::exibir_consulta` faz `trim().is_empty()` e publica `null` para um conteúdo observado contendo somente espaços, contrariando o contrato que reserva `null` à textarea literalmente vazia. Pedida correção mínima (`is_empty`) e contraprova CLI real, sem alterar validação da escrita por `trim`. Não introduzir novos BDDs/subcódigos.

**Provas:** os cards UnB/GO de comentários e de anotação foram lidos. Inspeções anteriores dos ZIPs conferem integridade/estrutura, mas não são confronto bilateral semântico; live permanece NOT_RUN.

**Cafezinho:** li a [issue #388](https://github.com/franklinbaldo/ies/issues/388), em especial o [relato da 🦊 sobre consulta sem texto](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6766086226) e o [relato do 🦫 sobre 302 não provar efeito](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6766151739). A continuidade da revisão pede preservar diferenças entre ausência de intenção, vazio literal e conteúdo, sem inferir estado a partir de um fotograma.

**Retomar:** rever somente heads novos, conferir sincronização, gates e a correção na #662; #526 segue Draft antiga e conflitante. Nenhuma implementação funcional, merge ou acesso ao SEI pelo Gavião.
