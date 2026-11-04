---
type: SEIObservation
title: Ensaio live de aberto_em e da natureza dos documentos em processo NUP na UnB
description: PASS/FAIL/NOT_RUN de aberto_em (lista, atribuição, anexado) e de icone/cancelado dos documentos, sem requisição a mais, 04/11/2026
---

# Ensaio live de `aberto_em` e da natureza dos documentos — 04/11/2026

Situação e natureza ([contrato](../../contracts/cli/processo-obter-situacao.md)),
binário em release do ramo `slice-situacao`, sessão e índice já existentes
(nenhum login), UnB 4.0.9 de treinamento, quatro execuções espaçadas de
25 s ou mais, todas com `--sem-andamentos --sem-propriedades
--sem-marcadores`. Só formas, contagens e conteúdo público das telas.

## Resultado

| Caso | Resultado |
| --- | --- |
| Processo aberto só na Unidade1 (frio, `--json`) | **PASS**: `pesquisa_rapida`, 3 req. (o mesmo orçamento do caminho 3), `aberto_em` com 1 unidade (`Unidade1`, sem atribuição), 1 documento `documento_interno`, `cancelado: false` |
| O mesmo, quente | **PASS**: `indice_link`, 1 req., mesma saída |
| Processo anexado (23106.001554/2026-99) | **PASS**: 3 req., `anexado_a` 23106.001559/2026-11, `aberto_em: null`, motivo `raiz_publica_o_vinculo_de_anexacao`, `aberto_em` primeiro em `nao_lidos` |
| Texto humano, outro processo (23106.001850/2026-90, quente) | **PASS**: 1 req., linha `Aberto em: Unidade1 (atribuído a aluno2), Unidade2.`, forma plural com atribuição por unidade |
| Documento cancelado ao vivo | NOT_RUN: nenhum processo da UnB à mão com cancelado; coberto pela captura de GO (teste sintético e teste local temporário) |
| GO e PEN1 ao vivo | NOT_RUN: GO por WAF e ritmo, PEN1 fora do escopo desta rodada; as formas de ambas vêm das capturas (offline) |

Nenhuma saída trouxe URL, `infra_hash` ou cookie. As contagens de
requisições são as do [contrato da árvore](../../contracts/cli/processo-obter-arvore.md):
a leitura nova não somou nenhuma.

## Observações

- A forma plural da UnB com atribuição (`Processo aberto nas unidades:` e
  ` (atribuído para …)` numa das linhas) só estava nas capturas de GO;
  ao vivo ela apareceu também na UnB e foi lida.
- Os testes de orçamento de `processo_obter` passaram sem mudança.
