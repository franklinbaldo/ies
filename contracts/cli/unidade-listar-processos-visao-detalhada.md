---
type: Contract
title: Listar processos na visão detalhada
description: ies unidade listar-processos [--pagina N] lê a visão D, publica colunas e categoria observada e deixa a tela como foi encontrada
---

# `ies unidade listar-processos` na visão detalhada (Refs #57)

Complementa o [contrato de `unidade.listar_processos`](unidade-listar-processos.md).
Decisão do proprietário (02/11/2026): o comando lê a **visão detalhada**
(`hdnTipoVisualizacao=D`, tabela `tblProcessosDetalhado`), que publica numa
lista só os processos de Recebidos e Gerados (total = soma das duas) e as
colunas configuradas no [Nível de detalhe](unidade-colunas.md). As listas
com categoria completa continuam em `ies unidade recebidos|gerados`
([listas](unidade-recebidos-gerados.md)).

## Saída

Um **array com uma lista** na [forma única](unidade-recebidos-gerados.md),
mais `colunas_ausentes`:

```json
[{"unidade": "TESTE", "origem": "web", "categoria": null,
  "total_declarado": 332, "faixa_observada": {"inicio": 1, "fim": 100},
  "colunas_ausentes": ["anotacao", "especificacao", "prioridade",
                       "observacao", "ultima_movimentacao", "marcadores"],
  "processos": [{"id": "42", "numero": "000042/2026", "visualizado": true,
                 "sigiloso": false, "categoria": "recebidos",
                 "colunas": {"atribuicao": "fulano", "tipo": "Ofício",
                             "interessados": ["Interessado A", "Interessado B"]}}]}]
```

- `categoria` da **lista** é `null`: D é modo de exibição, não categoria.
- `categoria` do **processo** é `"recebidos"` ou `"gerados"` quando o `id`
  apareceu nas tabelas `tblProcessosRecebidos`/`tblProcessosGerados` da
  **mesma** tela R obtida pelo GET deste comando; senão `null`, que quer
  dizer **"não observada"**, nunca "não tem". A D não traz marcador de
  categoria por linha (verificado nas capturas de UnB, PEN1 e GO); a
  informação só existe porque a R foi lida antes, sem requisição a mais.
  Sessão já na D: todos `null`.
- `colunas`: objeto com as chaves estáveis das colunas **presentes no
  cabeçalho** (`atribuicao`, `anotacao`, `tipo`,
  `especificacao`, `prioridade`, `interessados`, `observacao`,
  `controle_de_prazo`, `para_devolver`, `aguardando_retorno`,
  `ultima_movimentacao`, `marcadores`; ordem alfabética no JSON). Texto da célula com espaços
  colapsados (inclusive `&nbsp;`); célula vazia é `null`. `interessados` é
  um **array** com o texto de cada `div.divRotuloItemCelula` (itens
  separados pelo HTML, PEN1); célula vazia é `[]`. As demais colunas são
  texto único: `marcadores` não teve item observado (texto, até haver
  evidência). Colunas sem rótulo (seleção, ícones, ações) não entram.
- `colunas_ausentes`: chaves das **nove** colunas configuráveis de
  cabeçalho de texto que não vieram no cabeçalho; `[]` quando todas
  estão presentes (máximo = 9 + Processo, decisão do proprietário).
  Prazo, devolver e aguardando têm cabeçalho só de ícone e não contam
  aqui. A saída humana sugere marcá-las no Nível de detalhe; `ies unidade
  colunas` mostra a configuração (não há comando de escrita).

Recusas da tabela D, além das do contrato da operação:

| Fato observado | Subcódigo |
| --- | --- |
| Cabeçalho com rótulo fora de Processo e das 12 colunas | `cabecalho_da_visao_detalhada_nao_reconhecido` |
| Mesma coluna duas vezes no cabeçalho | `coluna_da_visao_detalhada_repetida` |
| `data-label` da célula diferente do cabeçalho da coluna | `rotulo_da_celula_detalhada_diferente_do_cabecalho` |
| Interessados com texto fora de `divRotuloItemCelula` | `itens_da_celula_detalhada_nao_separados` |
| Itens `divItemCelula` em coluna de texto único | `itens_nao_previstos_na_celula_detalhada` |

Rótulo do cabeçalho: texto de `div.infraDivRotuloOrdenacao` (ou do `th`);
sem texto, o `aria-label` do ícone (SEI 5.0.x) ou o argumento de
`infraTooltipMostrar('…')` (UnB 4.0.9). Cabeçalho sem rótulo fora de
`infraDivRotuloOrdenacao` é coluna sem rótulo; com o `div` e sem rótulo,
recusa.

## Sequência, orçamento e restauração

O comando **deixa a tela como a encontrou**. Requisições, sem retry:

| Tela no GET | `--pagina` | Requisições |
| --- | --- | --- |
| R | ausente ou igual à página lembrada da D | 1 GET + 2 POST (D, R) |
| R | N diferente da página lembrada da D | 1 GET + 3 POST (D, página N, R) |
| D | ausente ou igual à página exibida | 1 GET |
| D | N diferente da página exibida | 1 GET + 2 POST (página N, volta à página exibida) |

1. **GET** do Controle (travas existentes). Visão nem R nem D:
   `visao_do_controle_nao_reconhecida`.
2. **R:** lê as tabelas R (mesmas recusas da caixa, unidade obrigatória).
   `--pagina N` com N maior que `max(1, recebidos + gerados)` declarados é
   `pagina_fora_do_intervalo` **sem POST** (limite seguro: o tamanho da
   página da D só é conhecido depois de vê-la). **POST D** derivado de
   `frmProcedimentoControlar`, como `trocarVisualizacao('D')`: só
   `hdnTipoVisualizacao` muda. A resposta precisa ser D, da mesma unidade.
3. A D mostra a **sua** página lembrada (independente das páginas da R).
   Tamanho e número de páginas vêm de `hdnDetalhadoPaginaAtual` e da faixa,
   como em `--pagina N` da R. N fora do intervalo: `pagina_fora_do_intervalo`
   antes do POST de página (vindo da R, já depois do POST D, que é
   restaurado). D sem `hdnDetalhadoPaginaAtual` (UnB 4.0.9, sem paginação)
   só aceita a página 1, e só se a faixa cobre o total.
4. **POST página N** a partir do formulário da D, com
   `hdnDetalhadoPaginaAtual = N-1` (a R não publica esse campo; não é
   inventado). A resposta precisa manter unidade, visão D, total e
   `hdnDetalhadoHashCriterios`, e trazer exatamente a faixa da página N.
5. **Restauração.** Vindo da R: **POST R** a partir do formulário da última
   D obtida; a resposta precisa ser R com a mesma unidade, os mesmos
   critérios (`hdnRecebidosHashCriterios`, `hdnGeradosHashCriterios`) e as
   mesmas páginas (`hdnRecebidosPaginaAtual`, `hdnGeradosPaginaAtual`) do
   GET. Sessão já na D e página movida: POST da página exibida antes
   (`hdnDetalhadoPaginaAtual` original), com a faixa original.
   Vindo da R, a página lembrada da D **não** é restaurada (custaria mais
   um POST, e não é visível na R): fica na página N.

A restauração é tentada também quando um passo depois da troca falha
(leitura da D, N fora do intervalo, POST de página); se ela passa, o erro
publicado é o do passo. Se o próprio POST D falha, não há formulário D de
onde restaurar: o erro é o do POST e o estado da sessão fica por conferir
(GET manual).

## Falha da restauração: fechado, sem lista

Se a restauração falha, o comando **não publica a lista** (falha fechada,
escolha documentada na [decisão](../../docs/decisoes/unidade-listar-processos.md)):

| Fato | Subcódigo | Campos a mais |
| --- | --- | --- |
| POST R falhou ou a resposta não confere | `visao_do_controle_nao_restaurada` | `visao_deixada: "D"`, `causa` |
| Volta à página da D falhou ou não confere | `pagina_da_visao_detalhada_nao_restaurada` | `visao_deixada: "D"`, `pagina_deixada`, `causa` |

`causa` é o subcódigo da falha da restauração (por exemplo
`status_http_de_erro_do_servidor`, `visao_do_controle_nao_trocada_apos_envio`,
`paginas_da_visao_resumida_alteradas_apos_restaurar`,
`criterios_da_visao_resumida_alterados_apos_restaurar`,
`pagina_da_visao_detalhada_alterada_apos_restaurar`). Recusa HTTP publica
`status_http` e `mensagem_do_servidor`, como no [contrato da CLI](../cli.md).
Sem retry.

```json
{"codigo":"fonte_invalida","subcodigo":"visao_do_controle_nao_restaurada",
 "erro":"a tela do Controle ficou na visão detalhada: a restauração da resumida falhou",
 "retentavel":false,"visao_deixada":"D","causa":"status_http_de_erro_do_servidor",
 "status_http":500,"mensagem_do_servidor":"Erro."}
```

## Demais recusas da troca

| Fato | Subcódigo |
| --- | --- |
| 302 ao trocar de visão | `redirecionamento_ao_trocar_visao_do_controle` |
| Resposta da troca em outra visão | `visao_do_controle_nao_trocada_apos_envio` |
| Unidade diferente na resposta da troca | `unidade_alterada_ao_trocar_visao_do_controle` |
| `hdnDetalhadoPaginaAtual` ilegível ou repetido | `pagina_atual_da_visao_detalhada_ilegivel` |
| Índice incompatível com a faixa | `pagina_atual_da_visao_detalhada_incompativel_com_a_faixa` |
| Mais de uma página sem `lnkDetalhado*`/`selDetalhadoPaginacao*` | `paginador_da_visao_detalhada_nao_publicado` |
| Critério da D diferente após paginar | `criterios_da_visao_detalhada_alterados_apos_paginar` |
| Total ou faixa diferentes após paginar | `total_declarado_alterado_apos_paginar`, `faixa_da_resposta_diferente_da_pagina_solicitada` |

Derivação, envio, login, MIME, charset e status seguem a
[submissão derivada](../../docs/arquitetura/0002-submeter-formulario-observado.md)
e o [transporte](../cli.md). Efeitos avaliados:
`AlteraEstadoDaSessao::PreferenciaDeVisualizacao` (troca; na PEN1 um login
novo abriu na D) e `PaginaOuFiltroLembrado` (página da D).

## Opções

`--saida ARQUIVO` exporta o mesmo array, depois das verificações e da
restauração; `processo NUP --resumo --caixa-local` o lê (categoria por
processo preservada, `colunas` ignoradas). `--pagina-arquivo` foi
**removida** (erro de uso, saída 2): unia capturas privadas sem POST e não
tem semântica junto da troca de visão, cujas telas são de outra geração.
Capturas continuam legíveis offline por `ies replay`.

Golden: [`caixa-detalhada-v2.json`](../../tests/fixtures/caixa-detalhada-v2.json) (chaves em ordem alfabética).
Ensaio offline com HTML sintético modelado nas capturas de UnB, PEN1 e GO,
e leitura estrutural das próprias capturas privadas (sem copiar valores):
D da UnB (59 linhas, ícones pelo tooltip), PEN1 (páginas 1 e 2, Interessados
até 6 itens) e GO (12 colunas, `colunas_ausentes: []`); derivação da troca
nos formulários R e D das três.
Ensaio live: **NOT_RUN** (roteiro na [validação de 02/11/2026](../../docs/validacao-live/2026-11-02-leitura-controle.md)).
