---
type: SEIObservation
title: Replay de conformidade do acervo público pelos leitores de processo enviar
description: Link da barra, tela Enviar Processo, consulta do autocomplete, resposta do envio e regra de verificação por aberto_em nas 26 cassettes com procedimento_enviar, offline, 04/11/2026
---

# Replay do acervo — processo enviar — 04/11/2026

As respostas gravadas das 26 cassettes públicas que registram
`procedimento_enviar` passaram pelos **mesmos leitores** de
`ies processo enviar`, offline
(`cargo test --bin ies replay_de_processo_enviar -- --ignored`, ZIPs em
`IES_ACERVO_CACHE`, cada um conferido pelo card). Nenhuma requisição ao
SEI. Contrato em [processo enviar](../../contracts/cli/processo-enviar.md).

Leitores: o link de enviar na barra de cada árvore, cruzado com a unidade
de `lnkInfraUnidade` da última tela do processo em `aberto_em`; a tela
"Enviar Processo" conferida e a consulta derivada do script; a resposta de
cada consulta (as feitas depois da tela de envio e as de outra tela, a do
grupo de envio); o 302 do POST; e, por escrita, a regra de verificação: a
unidade nova do `aberto_em` relido é escolhida pela sigla entre os itens da
última consulta, e o POST é derivado da tela gravada com ela.

| Leitor | Classe | unb | ufrj | go | pen1 | pen2 | pen3 | pen4 |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| link na barra | não oferece; unidade fora de `aberto_em` (recusa) | 9 | 8 | 6 | 7 | 6 | 7 | 6 |
| link na barra | oferece; unidade em `aberto_em` (segue) | 102 | 100 | 25 | 29 | 25 | 28 | 23 |
| resposta da consulta | da tela de envio: um item | 7 | 0 | 0 | 6 | 4 | 6 | 4 |
| resposta da consulta | da tela de envio: vários itens | 0 | 5 | 4 | 0 | 0 | 0 | 0 |
| resposta da consulta | de outra tela: um item | 0 | 0 | 0 | 2 | 0 | 0 | 0 |
| resposta do envio | 302 esperado | 5 | 4 | 3 | 5 | 3 | 4 | 3 |
| tela de envio | conferida (forma 4.0, órgão ``); consulta derivada | 7 | 0 | 0 | 7 | 0 | 0 | 0 |
| tela de envio | conferida (forma 4.1/5.0, órgão ``); consulta derivada | 0 | 5 | 4 | 0 | 4 | 6 | 4 |
| verificação | 0 unidades novas em `aberto_em` | 0 | 0 | 0 | 1 | 0 | 0 | 0 |
| verificação | 2 unidades novas em `aberto_em` | 0 | 0 | 0 | 1 | 0 | 0 | 0 |
| verificação | verificada: destino da consulta aberto, origem mantida; POST derivado | 5 | 4 | 3 | 3 | 3 | 4 | 3 |

## Leitura

- As 37 telas foram conferidas, nas duas formas; o órgão efetivo é sempre
  "Todos" (`""`), e a consulta é derivada em todas.
- As 38 consultas destas cassettes (36 depois da tela de envio) são XML
  na forma observada (nenhuma recusa).
- Os 27 POST responderam o 302 esperado.
- Os 25 envios a uma unidade: o destino relido é um item da consulta e o
  POST deriva da tela; a origem foi mantida em todos (o roteiro do Todos
  envia com "manter aberto"). O envio **sem** manter aberto não foi
  gravado.
- As 2 exceções são o grupo de envio do PEN1 (duas unidades num POST, sem
  consulta; e o reenvio às mesmas unidades, sem unidade nova), fora do
  escopo; o reenvio é o caso que o comando recusa antes de escrever
  (`processo_ja_aberto_no_destino`).

## Busca do destino (GO) — rodada de 04/11/2026, à noite

Releitura com o ramo `slice-enviar-go`: a tabela acima saiu **idêntica**
(nenhuma regressão), e o replay do 302 de `processo concluir` também (32
no GO, todos relativos e esperados). Um segundo replay
(`cargo test --bin ies replay_da_busca_do_destino -- --ignored`) lê as 34
cassettes com a tela de envio ou a consulta (de qualquer tela): as opções
do `selOrgao` (`orgaos` da tela conferida) e quantas o filtro pelo prefixo
escolheria (`orgao_da_sigla`); cada resposta, se veio logo depois de uma
vazia (a forma do termo alargado), e quantas siglas com `/` da resposta a
regra exata escolhe só por elas. O corpo das consultas não foi gravado:
termo e órgão de cada uma não são conhecidos.

| Leitor | Classe | unb | ufrj | go | pen1 | pen2 | pen3 | pen4 |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| resposta da consulta | depois de uma vazia: 2 a 49 itens; siglas com `/` escolhidas pela exata: 39 de 39 | 0 | 0 | 2 | 0 | 0 | 0 | 0 |
| resposta da consulta | primeira ou depois de não vazia: 2 a 49 itens; siglas com `/` escolhidas pela exata: 1 de 1 | 0 | 0 | 8 | 0 | 0 | 0 | 0 |
| resposta da consulta | primeira ou depois de não vazia: 2 a 49 itens; siglas com `/` escolhidas pela exata: 4 de 4 | 0 | 7 | 0 | 0 | 0 | 0 | 0 |
| resposta da consulta | primeira ou depois de não vazia: nenhum item; siglas com `/` escolhidas pela exata: 0 de 0 | 0 | 0 | 2 | 0 | 0 | 0 | 0 |
| resposta da consulta | primeira ou depois de não vazia: um item; siglas com `/` escolhidas pela exata: 0 de 0 | 11 | 2 | 0 | 12 | 8 | 10 | 8 |
| resposta da consulta | primeira ou depois de não vazia: um item; siglas com `/` escolhidas pela exata: 1 de 1 | 0 | 0 | 2 | 0 | 0 | 0 | 0 |
| selOrgao | 3 opções; 1 órgãos escolhíveis pelo prefixo; órgão da tela `` | 7 | 0 | 0 | 0 | 4 | 6 | 4 |
| selOrgao | 4 opções; 2 órgãos escolhíveis pelo prefixo; órgão da tela `` | 0 | 5 | 0 | 7 | 0 | 0 | 0 |
| selOrgao | 64 opções; 62 órgãos escolhíveis pelo prefixo; órgão da tela `` | 0 | 0 | 4 | 0 | 0 | 0 | 0 |
- O `selOrgao` do GO tem 64 opções e 62 órgãos escolhíveis (rótulos
  únicos); as outras instalações, 1 ou 2.
- As 2 respostas "depois de uma vazia" são as 39 GECONT do grupo de envio
  do GO; cada uma das 39 siglas é escolhida só pela sigla exata, e nenhuma
  é `ABC/…`. Nenhuma resposta tem 50 ou mais itens: o corte é só do
  Todos.
