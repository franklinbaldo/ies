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

- `categoria`: `recebidos` ou `gerados`; na visão detalhada, `null`.
  A visão detalhada (`hdnTipoVisualizacao=D`) é um **modo de exibição**
  da sessão, não uma categoria do SEI: sua tabela única mistura as duas
  listas sem indicar a categoria de cada linha, e o IES não a infere.
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
| `ies unidade listar-processos [--pagina N]` | **array** `[lista D]`, `categoria: null` ([visão detalhada](unidade-listar-processos-visao-detalhada.md)) | 1 GET + 0 a 3 POST |
| `ies unidade recebidos` | **uma** lista, `categoria: "recebidos"` | 1 GET (sessão na D: + 2 POST) |
| `ies unidade gerados` | **uma** lista, `categoria: "gerados"` | 1 GET (sessão na D: + 2 POST) |
| `ies unidade recebidos\|gerados --pagina N` | uma lista, faixa da página N | 1 GET + 0 ou 1 POST (sessão na D: + 2 POST) |

Com `--pagina N`, N fora do intervalo é `pagina_fora_do_intervalo`
**antes** de qualquer POST; N igual à página mostrada não faz POST. A
sequência e as recusas estão nas [páginas da caixa](unidade-listar-processos-paginas.md).

## Sessão na visão detalhada: troca e restauração

`recebidos` e `gerados` precisam da visão R. Se o GET devolve a D (PEN1 e
GO lembram a D na sessão; na PEN1 até um login novo abriu nela), o comando
troca para R e **deixa a tela como a encontrou** (decisão do proprietário,
02/11/2026: utilidade em vez de recusa):

1. GET (tela D, unidade conferida).
2. **POST R** a partir do formulário da D (`trocarVisualizacao('R')`); a
   resposta precisa ser R da mesma unidade.
3. Com `--pagina N`: o plano e o POST do paginador da R descritos nas
   [páginas da caixa](unidade-listar-processos-paginas.md), a partir da R
   obtida. N fora do intervalo é recusado antes desse POST, mas depois do
   POST R (que é restaurado).
4. **POST D** a partir do formulário da última R obtida; a resposta precisa
   ser D da mesma unidade, com a mesma `hdnDetalhadoPaginaAtual` da tela do
   GET (a página lembrada da D não muda).

Orçamento: 1 GET + 2 POST (R, D); com `--pagina N` diferente da página
mostrada, 1 GET + 3 POST (R, página, D). A página da categoria pedida fica
lembrada, como na sessão R. A restauração é tentada também se um passo
depois do POST R falha. Falha da restauração: `visao_do_controle_nao_restaurada`,
com `visao_deixada: "R"`, `causa`, `status_http` e `mensagem_do_servidor`
quando houver, **sem lista** (as mesmas regras da
[visão detalhada](unidade-listar-processos-visao-detalhada.md)).

`ies replay CENARIO` (caixa resumida) usa o array `[recebidos, gerados]`;
`listar-processos --saida`, o array `[lista D]`.
`processo NUP --resumo --caixa-local` lê esse array: `[recebidos, gerados]`
ou, exportado da visão D, `[lista com categoria null]`, cujas ocorrências
saem com `categoria: null`. Uma exportação no formato anterior (objeto
com `recebidos`/`gerados`) ou a lista única `categoria: "detalhada"`
gravada pela `main` da PR #146 (pré-1.0, sem alias) é recusada com
`formato_do_arquivo_da_caixa_invalido`. Recusas novas da importação:

| Fato no arquivo | Subcódigo |
| --- | --- |
| Array diferente de `[recebidos, gerados]` e de `[null]` | `listas_do_arquivo_da_caixa_nao_sao_recebidos_e_gerados` |
| Lista sem categoria: total diferente das linhas | `total_da_lista_sem_categoria_do_arquivo_nao_comprovado` |
| Lista sem categoria: `id` inválido ou repetido | `id_da_lista_sem_categoria_do_arquivo_invalido`, `id_da_lista_sem_categoria_do_arquivo_repetido` |
| Lista sem categoria: `numero` inválido | `numero_da_lista_sem_categoria_do_arquivo_invalido` |
| Listas com `unidade` diferentes | `unidade_das_listas_do_arquivo_da_caixa_diferente` |
| Listas com `origem` diferentes | `origem_das_listas_do_arquivo_da_caixa_diferente` |

## Opções

| Opção | `listar-processos` | `recebidos`/`gerados` |
| --- | --- | --- |
| `--saida ARQUIVO` | exporta o array | exporta a lista |
| `--pagina-arquivo HTML` | **removida** | não existe |
| `--pagina N` | página da visão D | página da tabela R |
| `--categoria T` | **removida** | — (o subcomando é a categoria) |
| `--json`, `--unidade` (globais) | mantidas | mantidas, mesma conferência |

`--categoria` e `--pagina-arquivo` em `listar-processos` são erros de uso
do Clap (saída 2), sem leitura de configuração nem requisição. Contrato
provisório anterior a 1.0: **sem alias de compatibilidade**.

## Recusas da categoria

`recebidos` e `gerados` aplicam à sua tabela as mesmas recusas da caixa
(total, faixa, paginador sem intervalo) e exigem `lnkInfraUnidade`:
sem ele, `identificacao_da_unidade_ausente_no_controle`. A outra tabela
continua interpretada e verificada estruturalmente pela leitura da tela.
`--unidade SIGLA` diferente do rótulo publicado falha como em
`listar-processos`.
