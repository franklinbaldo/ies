---
type: SEIObservation
title: Ensaio live da colheita do Controle no índice e do passo 1b de processo NUP
description: PASS/FAIL/NOT_RUN da colheita sem requisição extra no login e da abertura pelo link do trabalhar colhido, UnB e PEN1, 03/11/2026
---

# Ensaio live da colheita do Controle — 03/11/2026

Colheita do Controle no [índice local](../arquitetura/0003-indice-local.md)
e passo 1b de [`ies processo NUP`](../../contracts/cli/processo-obter-arvore.md),
binário em release do ramo do PR, índice novo por instalação, ritmo
espaçado. Só contagens abaixo.

## Resultado

| Caso | UnB 4.0.9 | PEN1 4.0.12 |
| --- | --- | --- |
| `ies login --forcar` | **PASS**: 5 req. (as mesmas de antes); índice com 61 processos, 61 links de `trabalhar`, pesquisa e pesquisa rápida | **PASS**: 6 req.; 200 processos e 200 links |
| `processo NUP --sem-andamentos` de processo colhido | **PASS**: `indice_trabalhar`, 2 req. | **PASS**: 2 req. |
| Repetição com andamentos | **PASS**: `indice_link`, 2 req. (árvore + histórico) | **PASS** |
| Processo gerado colhido, com andamentos | **PASS**: `indice_trabalhar`, 3 req. | **PASS**: 3 req. |
| `ies login` com sessão válida | **PASS**: 1 req. (sem mudança) | **PASS** |

Em todos os PASS o NUP publicado é o pedido. A colheita não acrescentou
requisição a nenhum comando.

## Observações

- A colheita lê só a página do Controle que o comando já obteve; links
  de gerações antigas ficam no banco sem uso (as consultas são pela
  geração atual).
- GO, UFRJ, PEN2–4: NOT_RUN nesta rodada.
