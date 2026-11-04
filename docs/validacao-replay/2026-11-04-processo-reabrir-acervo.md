---
type: SEIObservation
title: Replay de conformidade do acervo público pelos leitores de processo reabrir
description: Ação de reabrir da barra cruzada com a regra de decisão, URL de reabrir na tela do nó (função ou variável), resposta do GET de reabrir e verificação por aberto_em nas 190 cassettes com procedimento_concluir ou procedimento_reabrir, offline, 04/11/2026
---

# Replay do acervo — processo reabrir — 04/11/2026

As respostas gravadas das 190 cassettes públicas que registram
`procedimento_concluir` ou `procedimento_reabrir` passaram pelos **mesmos
leitores** de `ies processo reabrir`, offline
(`cargo test --bin ies replay_de_processo_reabrir -- --ignored`, ZIPs em
`IES_ACERVO_CACHE`, cada um conferido pelo card). Nenhuma requisição ao
SEI. Contrato em [processo reabrir](../../contracts/cli/processo-reabrir.md).

Leitores: a ação de reabrir na barra de cada árvore
(`procedimento_visualizar`), cruzada com a regra de decisão do contrato (a
unidade de `lnkInfraUnidade` da última tela do processo em `aberto_em`); a
URL de reabrir em cada tela do nó (`arvore_visualizar`); a resposta de
cada GET de `procedimento_reabrir` pela mesma função do comando; e, por
escrita, a verificação: a unidade fora de `aberto_em` antes e dentro dele
na primeira árvore do mesmo processo depois.

| Leitor | Classe | unb | ufrj | go | pen1 | pen2 | pen3 | pen4 |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| URL na tela do nó | `funcao` | 104 | 0 | 0 | 56 | 0 | 0 | 0 |
| URL na tela do nó | `variavel` | 0 | 88 | 39 | 0 | 31 | 27 | 34 |
| ação de reabrir na barra | não oferece; unidade em `aberto_em` (nada a fazer) | 290 | 304 | 202 | 177 | 185 | 176 | 184 |
| ação de reabrir na barra | não oferece; unidade ou situação não lida | 4 | 4 | 4 | 4 | 4 | 4 | 4 |
| ação de reabrir na barra | oferece; unidade fora de `aberto_em` (reabrir) | 36 | 42 | 36 | 34 | 30 | 32 | 33 |
| resposta da escrita | HTML 200 esperado | 1 | 2 | 2 | 1 | 1 | 2 | 2 |
| verificação | verificada (unidade da sessão aberta depois) | 1 | 2 | 2 | 1 | 1 | 2 | 2 |

Leitura: nenhuma recusa nem contradição. Reabrir aparece na barra sempre
com a unidade da sessão fora de `aberto_em`, nunca com ela dentro; com ela
dentro, a barra não oferece reabrir (o caso "nada a fazer"). Nas 28 árvores
em que a situação ou a unidade da sessão não foi lida (4 por instalação),
o comando recusaria antes de escrever. Toda tela do nó publica a
URL em exatamente uma forma: a função no 4.0 (UnB, PEN1), a variável no
4.1/5.0 (as outras cinco). As 11 escritas gravadas teriam sido aceitas
como resposta esperada e verificadas pela situação.
