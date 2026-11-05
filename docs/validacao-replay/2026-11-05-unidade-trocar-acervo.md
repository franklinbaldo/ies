---
type: SEIObservation
title: Replay de conformidade do acervo público pelos leitores de unidade trocar
description: Link da troca no Controle, tela Trocar Unidade, POST derivado, os dois Location da troca (a URL que o IES pediria igual à gravada) e o Controle da unidade nova, nas 32 cassettes com infra_trocar_unidade, offline, 05/11/2026
---

# Replay do acervo — unidade trocar — 05/11/2026

As respostas gravadas das cassettes públicas que registram
`infra_trocar_unidade` passaram pelos **mesmos leitores** de
`ies unidade trocar`, offline
(`cargo test --bin ies replay_de_unidade_trocar -- --ignored`, ZIPs em
`IES_ACERVO_CACHE`, cada um conferido pelo card). Nenhuma requisição ao
SEI. Contrato em [unidade trocar](../../contracts/cli/unidade-trocar.md).

Leitores: o link de troca de cada Controle reconhecido; cada tela Trocar
Unidade (leitura e conferência); o POST derivado para a primeira unidade
não marcada (só onde a tela oferece mais de uma); em cada POST gravado,
o id que o `Location` declara (o corpo não é gravado) entre as unidades
oferecidas e diferente do marcado; a URL que o IES pediria pelo `Location`
do POST e pelo do salto seguinte, comparada **byte a byte** com a URL que a
gravação pediu (o `msg` com `%20`; `http://` do GO pedido em `https://`); e
a sigla do Controle da unidade nova igual à da unidade escolhida.

32 cassettes lidas.

| Leitor | Instalação | Total | Lido | Recusado | Não aplicável |
|---|---|---:|---:|---|---|
| link da troca no Controle | unb | 40 | 40 | 0 | 0 |
| link da troca no Controle | ufrj | 12 | 12 | 0 | 0 |
| link da troca no Controle | go | 14 | 14 | 0 | 0 |
| link da troca no Controle | pen1 | 1 | 1 | 0 | 0 |
| link da troca no Controle | pen2 | 1 | 1 | 0 | 0 |
| link da troca no Controle | pen3 | 5 | 5 | 0 | 0 |
| link da troca no Controle | pen4 | 5 | 5 | 0 | 0 |
| link da troca no Controle | **todas** | 78 | 78 | 0 | 0 |
| tela Trocar Unidade | unb | 52 | 52 | 0 | 0 |
| tela Trocar Unidade | ufrj | 15 | 15 | 0 | 0 |
| tela Trocar Unidade | go | 48 | 48 | 0 | 0 |
| tela Trocar Unidade | pen1 | 1 | 1 | 0 | 0 |
| tela Trocar Unidade | pen2 | 1 | 1 | 0 | 0 |
| tela Trocar Unidade | pen3 | 36 | 36 | 0 | 0 |
| tela Trocar Unidade | pen4 | 36 | 36 | 0 | 0 |
| tela Trocar Unidade | **todas** | 189 | 189 | 0 | 0 |
| POST derivado | unb | 52 | 52 | 0 | 0 |
| POST derivado | ufrj | 15 | 15 | 0 | 0 |
| POST derivado | go | 48 | 48 | 0 | 0 |
| POST derivado | **todas** | 115 | 115 | 0 | 0 |
| id do 302 entre as oferecidas, não a marcada | unb | 8 | 8 | 0 | 0 |
| id do 302 entre as oferecidas, não a marcada | ufrj | 8 | 8 | 0 | 0 |
| id do 302 entre as oferecidas, não a marcada | go | 6 | 6 | 0 | 0 |
| id do 302 entre as oferecidas, não a marcada | **todas** | 22 | 22 | 0 | 0 |
| Location do POST (URL = a pedida) | unb | 8 | 8 | 0 | 0 |
| Location do POST (URL = a pedida) | ufrj | 8 | 8 | 0 | 0 |
| Location do POST (URL = a pedida) | go | 6 | 6 | 0 | 0 |
| Location do POST (URL = a pedida) | **todas** | 22 | 22 | 0 | 0 |
| Location do 1º salto (URL = a pedida) | unb | 8 | 8 | 0 | 0 |
| Location do 1º salto (URL = a pedida) | ufrj | 8 | 8 | 0 | 0 |
| Location do 1º salto (URL = a pedida) | go | 6 | 6 | 0 | 0 |
| Location do 1º salto (URL = a pedida) | **todas** | 22 | 22 | 0 | 0 |
| Controle da unidade nova | unb | 8 | 8 | 0 | 0 |
| Controle da unidade nova | ufrj | 8 | 8 | 0 | 0 |
| Controle da unidade nova | go | 6 | 6 | 0 | 0 |
| Controle da unidade nova | **todas** | 22 | 22 | 0 | 0 |

Leitura: nenhuma recusa. As 22 trocas gravadas (UnB 8, UFRJ 8, GO 6)
teriam sido seguidas e verificadas com 5 requisições cada. As PEN só
gravaram a tela com uma unidade oferecida (sem POST derivado). O replay
não prova o corpo do POST (não gravado), só que o derivado tem a forma do
formulário servido e de `selecionarUnidade()`.
