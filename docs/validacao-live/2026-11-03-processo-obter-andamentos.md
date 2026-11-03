---
type: SEIObservation
title: Ensaio live dos andamentos de processo NUP em três instalações de treinamento
description: PASS/FAIL/NOT_RUN do histórico resumido e completo, do índice quente e das projeções de ies processo NUP, 03/11/2026
---

# Ensaio live dos andamentos de `ies processo NUP` — 03/11/2026

Recorte 2 da #59 ([contrato](../../contracts/cli/processo-obter-andamentos.md)),
binário em release do ramo do PR, [índice local](../arquitetura/0003-indice-local.md)
novo por instalação. Ritmo espaçado (pausas de 2 a 3 s), depois do
bloqueio do WAF de GO na mesma tarde. Só formas e contagens abaixo.

## Resultado

| Caso | UnB 4.0.9 | PEN1 4.0.12 | GO 5.0.3 |
| --- | --- | --- | --- |
| Frio (`caixa` + histórico) | **PASS**: 4 req., 18 itens = legenda | **PASS**: 4 req., 12 itens, `resumido` | **PASS**: 4 req., 49 itens |
| Quente (`indice_link` + histórico guardado) | **PASS**: 2 req. | **PASS**: 2 req. | NOT_RUN |
| `--andamentos-completos` | **PASS**: 2 req. (a tela já servia `P`, POST dispensado) | **PASS**: 3 req., 27 itens, `completo` | NOT_RUN |
| GET simples depois do `P` | `completo` | **`completo`**: visão lembrada | `completo` (lembrada de ensaio anterior) |
| `--sem-andamentos` | **PASS**: 1 req., motivo `omitidos_por_sem_andamentos` | **PASS** | NOT_RUN |
| `--sem-andamentos --andamentos-completos` | **PASS**: saída 2 | **PASS**: saída 2 | NOT_RUN |
| Árvore sem link do histórico | NOT_RUN | NOT_RUN | NOT_RUN (visto no ensaio estrutural; não repetido para não forçar o WAF) |
| `bloqueio_de_seguranca_da_instalacao` | — | — | NOT_RUN (o bloqueio já tinha caído; só testes offline) |

Em todos os PASS o NUP publicado é o pedido e os itens lidos são iguais
ao total da legenda.

## Observações

- **A visão do histórico é lembrada pelo servidor.** Medido na PEN1:
  depois do POST com `hdnTipoHistorico=P`, um GET simples serviu `P`
  (27 itens em vez de 12). Na UnB e em GO o primeiro GET já veio `P`
  por ensaios anteriores. O efeito `PaginaOuFiltroLembrado` declarado
  está confirmado; o IES publica o `tipo` servido e não restaura `R`.
- **Paginação:** nenhuma até 75 linhas (GO); `hdnInfraNroItens` e
  `hdnInfraPaginaAtual` não mudam o tamanho nem a página (ablação na
  UnB). Históricos maiores: NOT_RUN.
