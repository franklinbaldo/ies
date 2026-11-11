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
 "total_declarado": 114, "faixa_observada": [1, 100],
 "processos": [{"id": "42", "numero": "000042/2026",
                "visualizado": true, "sigiloso": false}]}
```

- `categoria`: `recebidos` ou `gerados` nas listas da visão resumida; a
  visão detalhada não tem o campo na lista.
  A visão detalhada (`hdnTipoVisualizacao=D`) é um **modo de exibição**
  da sessão, não uma categoria do SEI: sua tabela única mistura as duas
  listas sem indicar a categoria de cada linha, e o IES não a infere.
- `total_declarado`: total da legenda (sempre número; legenda sem total
  falha com `TotalNaoDeclaradoNaLegenda`); `faixa_observada`: `[inicio, fim]`
  das posições lidas, ou `[]` com zero registros. Cobertura não confirmada
  pela legenda falha com `LegendaSemFaixaComTotalDiferenteDasLinhas`.
  Nunca `completa`, nunca `null`.
- `unidade`: rótulo publicado em `lnkInfraUnidade` (contexto observado,
  não eco de `--unidade`); `origem`: `web`, `web_com_paginas_privadas` ou,
  nos testes da caixa resumida, a instalação da cassette.
- Cada processo: `id`, `numero` (opaco), `visualizado`, `sigiloso`. Na
  visão detalhada os dois últimos são sempre booleanos; um link sem classe
  de estado reconhecida recusa a leitura (`EstadoDoLinkDoProcessoAusente`).

## Comandos

| Comando | Saída | Requisições |
| --- | --- | --- |
| `ies unidade listar-processos [--pagina N]` | **array** `[lista D]`, sem `categoria` na lista ([visão detalhada](unidade-listar-processos-visao-detalhada.md)) | 1 GET + 0 a 2 POST |
| `ies unidade recebidos` | **uma** lista, `categoria: "recebidos"` | 1 GET (sessão na D: + 1 POST R) |
| `ies unidade gerados` | **uma** lista, `categoria: "gerados"` | 1 GET (sessão na D: + 1 POST R) |
| `ies unidade recebidos\|gerados --pagina N` | uma lista, faixa da página N | 1 GET + 0 ou 1 POST (sessão na D: + 1 POST R) |

Com `--pagina N`, N fora do intervalo é `pagina_da_caixa_fora_do_intervalo`
**antes** de qualquer POST; N igual à página mostrada não faz POST. A
sequência e as recusas estão nas [páginas da caixa](unidade-listar-processos-paginas.md).

## Sessão na visão detalhada: troca para R, sem volta

`recebidos` e `gerados` precisam da visão R. Se o GET devolve a D (PEN1 e
GO lembram a D na sessão; na PEN1 até um login novo abriu nela), o comando
troca para R (decisão do proprietário, 02/11/2026: utilidade em vez de
recusa) e **fica na R** (decisão do proprietário, 02/11/2026, na
[decisão da operação](../../docs/decisoes/unidade-listar-processos.md):
restaurar custava requisições e o efeito residual é cosmético):

1. GET (tela D, unidade conferida).
2. **POST R** a partir do formulário da D (`trocarVisualizacao('R')`); a
   resposta precisa ser R da mesma unidade.
3. Com `--pagina N`: o plano e o POST do paginador da R descritos nas
   [páginas da caixa](unidade-listar-processos-paginas.md), a partir da R
   obtida. N fora do intervalo é recusado antes desse POST, mas depois do
   POST R (a sessão fica na R). A página N da categoria fica lembrada,
   como com a sessão já na R; `faixa_observada` diz qual é.

Orçamento: 1 GET + 1 POST (R); com `--pagina N` diferente da página
mostrada, 1 GET + 2 POST (R, página N). A recusa de página depois do POST
R tem a mesma mensagem da sessão já na R. Ensaio live de 04/11/2026
([relatório](../../docs/validacao-live/2026-11-04-unidade-paginacao.md)): **PASS** na UnB 4.0.9, na PEN1, na PEN3 e na PEN4
5.0.4, com e sem `--pagina N` e nas duas ordens, e na UFRJ 4.1.5 (uma
página, legenda `(1 registro):` aceita depois da correção); PEN2 (login
com 503, depois 403 do Cloudflare) **NOT_RUN**; GO 5.0.3 (login
interrompido pelo proxy na segunda tentativa) **PASS** na terceira ([GO e PEN2](../../docs/validacao-live/2026-11-04-unidade-paginacao-go-pen2.md)). Regime estável: depois da troca a
sessão está na R, e `recebidos|gerados` custa 1 GET (mais o POST de página,
se pedido). Uma falha é publicada como está, sem lista; não há POST de
volta. As recusas `visao_do_controle_nao_restaurada` e
`pagina_da_visao_resumida_nao_restaurada` da versão que restaurava a D
deixaram de existir.

A caixa resumida (golden [`caixa-v2.json`](../../tests/fixtures/caixa-v2.json),
conferido por teste unitário) é o array `[recebidos, gerados]`;
`listar-processos --saida`, o array `[lista D]`.
O leitor desses arquivos (`processo NUP --resumo --caixa-local`) foi
aposentado em 04/11/2026 com o `--resumo`
([processo.obter](processo-obter.md)); a exportação não muda.

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
sem ele, `identificacao_da_unidade_ausente_no_controle`. Tabela da
categoria ausente é `tabela_ausente`, salvo a lista vazia comprovada pela
estrutura (GO; [páginas da caixa](unidade-listar-processos-paginas.md)),
publicada com `total_declarado: 0`. A outra tabela
continua interpretada e verificada estruturalmente pela leitura da tela.
`--unidade SIGLA` diferente do rótulo publicado falha como em
`listar-processos`.

## Histórico de restauração

As recusas de restauração citadas neste contrato são históricas: foram
aposentadas pela decisão de #57 de deixar a sessão na visão/página
utilizada, sem requisições para restaurar a tela. Não são emissões ativas.
