---
type: SEIObservation
title: Ensaio live das propriedades de processo NUP na UnB e na PEN1
description: PASS/FAIL/NOT_RUN da leitura dos metadados pela barra da árvore, com as projeções e o índice quente, 04/11/2026
---

# Ensaio live das propriedades de `ies processo NUP` — 04/11/2026

Propriedades ([contrato](../../contracts/cli/processo-obter-propriedades.md)),
binário em release do ramo do PR, índice novo, sessão nova, ritmo espaçado.
Só formas e contagens abaixo.

## Resultado (UnB 4.0.9)

| Caso | Resultado |
| --- | --- |
| Padrão, índice quente | **PASS**: 3 req., `fonte: procedimento_alterar`, tipo, especificação, 2 interessados, 1 assunto, `publico`, data |
| `--sem-propriedades` | **PASS**: 2 req., motivo `omitidas_por_sem_propriedades` |
| `--sem-andamentos` | **PASS**: 2 req. |
| `--sem-andamentos --sem-propriedades` | **PASS**: 1 req. |
| Processo aberto só em outra unidade (com pastas) | **PASS**: `fonte: procedimento_consultar` (primeira leitura dessa tela pelo IES) |

O NUP e o id da tela conferiram em todos. Nenhum POST foi enviado.

## Observações

- A primeira chamada na UnB deu `falha_de_rede` (rede intermitente); a
  repetição passou.
- **PEN1 4.0.12:** primeiro NOT_RUN, porque a árvore do processo escolhido
  tinha um nó `DOCUMENTO_MOVIDO` então recusado; depois do recorte dos
  documentos movidos, **PASS** ([relatório](2026-11-04-processo-obter-movidos.md)).
- Processo restrito/sigiloso (hipótese legal servida): NOT_RUN.
