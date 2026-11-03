---
type: SEIObservation
title: Ensaio live das pastas da árvore de processo NUP na UnB
description: PASS/FAIL/NOT_RUN da abertura de pastas, da conferência por identidade e por contagem e do índice quente, 03/11/2026
---

# Ensaio live das pastas de `ies processo NUP` — 03/11/2026

Recorte 3 da #59 ([contrato](../../contracts/cli/processo-obter-pastas.md)),
binário em release do ramo do PR, índice local novo, ritmo espaçado.
Processos de treinamento da UnB 4.0.9 com pastas (os mesmos das cassettes
públicas do Todos). Só formas e contagens abaixo.

## Resultado

| Caso | Processo A (157 docs, 8 pastas) | Processo B (22 docs, 2 pastas) |
| --- | --- | --- |
| Frio (pesquisa rápida + abrir pastas) | **PASS**: 5 req., 157/157 em pasta, `por_contagem` em todas | **PASS**: 4 req., PASTA1 `mista` (19 + 1), PASTA2 `por_identidade` |
| Quente (link de abrir pastas no índice) | **PASS**: `indice_link`, 1 req. | **PASS**: 1 req. |
| Sessão nova, entrada por id | **PASS**: 4 req.; a árvore voltou com as pastas fechadas e foi reaberta | NOT_RUN |

Em todos os PASS o NUP publicado é o pedido, não restou nenhum `AGUARDE`
e a contagem por pasta é igual à declarada.

## Observações

- **Primeira versão da regra B falhou ao vivo.** A identidade
  (`id_rel_protocolo_protocolo` na barra do nó) varia por documento: barras
  vazias, barras com ações sem `comentario_listar` e barras com identidade
  convivem na mesma pasta. A regra adotada confere por identidade o que tem
  identidade e conta o resto ([decisão](../decisoes/processo-obter.md)).
- **O servidor não lembra as pastas abertas entre sessões**: depois de
  `abrir_pastas=1`, a árvore simples numa sessão nova veio fechada.
  Na mesma sessão: NOT_RUN.
- **Corrida no índice encontrada e corrigida no mesmo PR:** duas conexões
  preparando um banco novo podiam desligar o índice (troca para WAL
  recusada sem erro). O teste agora repete o cenário 12 vezes.
- Processos com pastas noutras instalações e processo anexado
  (`PROCESSO_ANEXADO`, ainda recusado): NOT_RUN.
