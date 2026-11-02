---
type: Contract
title: Listas Recebidos e Gerados da unidade
description: Forma única de lista da caixa e os comandos ies unidade recebidos e ies unidade gerados
---

# Listas Recebidos e Gerados (Refs #57)

Complementa o [contrato de `unidade.listar_processos`](unidade-listar-processos.md)
e as [páginas da caixa](unidade-listar-processos-paginas.md). A operação
continua sendo `unidade.listar_processos`: os três comandos leem a mesma
tela **Controle de Processos** e compartilham transporte, travas e
verificações.

## Duas listas independentes

Recebidos e Gerados são **duas listas independentes** do SEI: cada uma tem
paginador, total, faixa e página lembrada próprios
(`hdn{T}PaginaAtual`, `lnk{T}*`, `sel{T}Paginacao*`). Só compartilham a
resposta HTTP. A CLI as modela como listas separadas em toda saída
(decisão do proprietário, 02/11/2026).

## Forma única da lista

Toda saída JSON de caixa publica listas com a mesma forma, a mesma que
`--categoria` devolvia na PR #146:

```json
{"unidade": "TESTE", "origem": "web", "categoria": "recebidos",
 "total_declarado": 114, "faixa_observada": {"inicio": 1, "fim": 100},
 "processos": [{"id": "42", "numero": "000042/2026",
                "visualizado": true, "sigiloso": false}]}
```

- `categoria`: `recebidos`, `gerados` ou, na visão detalhada, `detalhada`.
- `total_declarado`: total da legenda; `faixa_observada`: `{inicio, fim}`
  das posições lidas, ou `null` sem linhas. Nunca `completa`.
- `unidade`: rótulo publicado em `lnkInfraUnidade` (contexto observado,
  não eco de `--unidade`); `origem`: `web`, `web_com_paginas_privadas` ou
  o cenário do replay.
- Cada processo: `id`, `numero` (opaco), `visualizado`, `sigiloso`. Na
  visão detalhada os dois últimos são `null` quando a classe do link não
  é reconhecida.

## Comandos

| Comando | Saída | Requisições |
| --- | --- | --- |
| `ies unidade listar-processos` | **array** `[recebidos, gerados]`, nessa ordem | 1 GET |
| `ies unidade recebidos` | **uma** lista, `categoria: "recebidos"` | 1 GET |
| `ies unidade gerados` | **uma** lista, `categoria: "gerados"` | 1 GET |
| `ies unidade recebidos\|gerados --pagina N` | uma lista, faixa da página N | 1 GET + 0 ou 1 POST |

Com `--pagina N`, N fora do intervalo é `pagina_fora_do_intervalo`
**antes** de qualquer POST; N igual à página mostrada não faz POST. A
sequência e as recusas estão nas [páginas da caixa](unidade-listar-processos-paginas.md).

Na visão detalhada (`hdnTipoVisualizacao=D`), `listar-processos` devolve
um array com **uma** lista `categoria: "detalhada"`; `recebidos` e
`gerados` falham com `tabela_ausente` e a categoria pedida, porque a tela
não publica a tabela da categoria. `--pagina N` não se aplica à visão D.

`ies replay CENARIO` (caixa) e a exportação `--saida` usam o mesmo array.
`processo NUP --resumo --caixa-local` lê esse array; uma exportação no
formato anterior (objeto com `recebidos`/`gerados`) é recusada com
`formato_do_arquivo_da_caixa_invalido`. Recusas novas da importação:

| Fato no arquivo | Subcódigo |
| --- | --- |
| Array diferente de `[recebidos, gerados]` | `listas_do_arquivo_da_caixa_nao_sao_recebidos_e_gerados` |
| Listas com `unidade` diferentes | `unidade_das_listas_do_arquivo_da_caixa_diferente` |
| Listas com `origem` diferentes | `origem_das_listas_do_arquivo_da_caixa_diferente` |

## Opções

| Opção | `listar-processos` | `recebidos`/`gerados` |
| --- | --- | --- |
| `--saida ARQUIVO` | exporta o array | exporta a lista |
| `--pagina-arquivo HTML` | mantida (une páginas privadas, 0 POST) | não existe |
| `--pagina N` | **removida** | mantida |
| `--categoria T` | **removida** | — (o subcomando é a categoria) |
| `--json`, `--unidade` (globais) | mantidas | mantidas, mesma conferência |

`--categoria` e `--pagina` em `listar-processos` são erros de uso do Clap
(saída 2), sem leitura de configuração nem requisição. Contrato
provisório anterior a 1.0: **sem alias de compatibilidade**.

## Recusas da categoria

`recebidos` e `gerados` aplicam à sua tabela as mesmas recusas da caixa
(total, faixa, paginador sem intervalo) e exigem `lnkInfraUnidade`:
sem ele, `identificacao_da_unidade_ausente_no_controle`. A outra tabela
continua interpretada e verificada estruturalmente pela leitura da tela.
`--unidade SIGLA` diferente do rótulo publicado falha como em
`listar-processos`.
