---
type: SEIObservation
title: Ensaio live dos documentos movidos e das propriedades de processo NUP na PEN1
description: PASS/FAIL/NOT_RUN da árvore com DOCUMENTO_MOVIDO, do destino aprendido e das propriedades, PEN1 4.0.12, 04/11/2026
---

# Ensaio live dos documentos movidos — 04/11/2026

> Nota (04/11/2026): a PEN1 deste ensaio já servia o SEI 5.0.4; "4.0.12" é a versão das cassettes de outubro ([evidência](../interfaces/instalacoes-e-versoes.md)).

Documentos movidos ([contrato](../../contracts/cli/processo-obter-arvore.md)),
binário em release do ramo do PR, índice novo, sessão nova, ritmo espaçado.
O processo é o mesmo que bloqueava as propriedades na PEN1. Só formas e
contagens abaixo.

## Resultado (PEN1 4.0.12)

| Caso | Resultado |
| --- | --- |
| Processo com `DOCUMENTO_MOVIDO`, padrão | **PASS**: `indice_trabalhar` (colhido no login), 4 req. (árvore, histórico, propriedades); 2 documentos, 1 movido com destino; 37 andamentos; propriedades por `procedimento_alterar` |
| Processo de destino pelo link do movido | **PASS**: `indice_trabalhar`, 2 req., NUP confere |

O NUP publicado é o pedido nos dois casos. Com isso as
[propriedades](2026-11-04-processo-obter-propriedades.md) também passam
na PEN1.

## Observações

- O link do destino traz `id_documento`; aberto pelo passo 1b, levou à
  árvore do processo de destino, conferida pela identidade.
- Documento movido de volta, ou dentro de pasta: NOT_RUN (recusa).
