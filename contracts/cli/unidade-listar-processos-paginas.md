---
type: Contract
title: Páginas da caixa da unidade
description: Página observada por padrão e opção --pagina N em ies unidade recebidos e gerados (a D em listar-processos)
---

# Páginas da caixa da unidade (Refs #57)

Complementa o [contrato de `unidade.listar_processos`](unidade-listar-processos.md)
e aplica a [regra geral de leituras paginadas](../cli.md) à caixa.

## Página observada, sem POST

Sem `--pagina`, `ies unidade recebidos` e `gerados` (sessão na R) fazem só
o GET existente e devolvem, por lista (Recebidos e Gerados), os
processos que a tela mostra, o `total_declarado` e a `faixa_observada`,
na [forma única](unidade-recebidos-gerados.md):

```json
{"categoria": "recebidos", "total_declarado": 114, "faixa_observada": {"inicio": 1, "fim": 100}, "processos": [...]}
```

- A faixa vem da legenda (`(114 registros - 1 a 100):`). Com legenda sem
  intervalo e linhas iguais ao total (`3 registros:`), a faixa é `1..total`;
  o singular `(1 registro):` (UFRJ) vale igual.
  Tabela com zero registros publica `faixa_observada: null`.
- No replay de páginas capturadas, a faixa é a união contígua das
  páginas recebidas, em qualquer ponto da lista.
- A faixa pode não começar em 1: o servidor lembra a página pedida na
  sessão. A saída não diz "primeira página".
- `paginas_do_controle_nao_obtidas` deixa de ser emitido por este comando.

Continuam recusas tipadas, antes de publicar ou exportar:

| Fato observado | Subcódigo |
| --- | --- |
| Legenda sem `registros` (total não declarado) | `total_nao_declarado_na_legenda` |
| Legenda sem intervalo e total diferente das linhas | `legenda_sem_faixa_com_total_diferente_das_linhas` |
| Paginador da tabela publicado e legenda sem intervalo | `paginador_da_tabela_sem_faixa_na_legenda` |
| Faixa inválida, alterada ou páginas não contíguas | `faixa_invalida`, `faixa_alterada`, `faixa_descontinua` |

Paginadores por tabela reconhecidos pelo `id`: `lnk{T}ProximaPagina*`,
`lnk{T}PaginaAnterior*`, `lnk{T}PrimeiraPagina*`, `lnk{T}UltimaPagina*` e
`sel{T}Paginacao*`, com `T` igual a `Recebidos` ou `Gerados`.
`hdn{T}PaginaAtual` **não** indica paginação: aparece também em telas
de uma página (UnB e UFRJ, 02/11/2026).

## Opção `--pagina N`

`ies unidade recebidos --pagina N` e `ies unidade gerados --pagina N`
pedem a página N (inteiro ≥ 1) de **uma** lista. Recebidos e Gerados têm
paginadores independentes, e cada ação de página do navegador altera só
um deles; por isso a página de uma tabela R só se pede nesses subcomandos,
que não aceitam `--pagina-arquivo`. A saída é a lista, com a faixa da
página obtida. `listar-processos --pagina N` pede a página da visão D
([visão detalhada](unidade-listar-processos-visao-detalhada.md)). Sessão na
D: a troca para R e a volta à D estão nas
[listas](unidade-recebidos-gerados.md); a sequência abaixo vale a partir
da tela R.

Sequência e orçamento:

1. GET do Controle, com as travas existentes. A tela precisa ser a visão
   resumida (`hdnTipoVisualizacao=R`).
2. Tamanho da página e número de páginas a partir da tela: se
   `hdn{T}PaginaAtual = 0`, tamanho = fim da faixa; se for `k > 0`,
   tamanho = `(inicio - 1) / k`, exato. Páginas = ⌈total / tamanho⌉.
   Faixa incompatível com o índice é recusa.
3. N fora de `1..=páginas` → `pagina_fora_do_intervalo`, **sem POST**.
   N igual à página mostrada → nenhum POST.
4. Senão, **um POST** derivado de `frmProcedimentoControlar`, como
   `infraAcaoPaginar('=', N-1, T)`: todos os campos ocultos como servidos,
   salvo `hdn{T}PaginaAtual = N-1`; `txtBloco` e os `sel*Paginacao*`
   com os valores servidos; caixas de seleção desmarcadas omitidas.
   Sem botão de envio (o JavaScript chama `submit()`).
5. Resposta: mesmas travas de HTTP, tamanho, charset e tela de login do GET;
   302 é recusa. A página obtida precisa ter a mesma unidade, visualização
   e critérios (`hdnRecebidosHashCriterios`, `hdnGeradosHashCriterios`) da
   tela de origem, o mesmo total e exatamente a faixa da página N.

Total: **1 GET + 0 ou 1 POST**. Sem retry, sem outra página, sem
restauração. Efeito avaliado: `AlteraEstadoDaSessao::PaginaOuFiltroLembrado`.
O GET seguinte pode mostrar a página N da tabela; a faixa publicada diz qual.
Usar só em instalação de treinamento autorizada. Ensaio live da sintaxe
`recebidos|gerados --pagina N` em 04/11/2026 ([relatório](../../docs/validacao-live/2026-11-04-unidade-paginacao.md)): PASS na
UnB, na PEN1, na PEN3 e na PEN4 (5.0.4, duas páginas por lista) e na UFRJ
4.1.5 (uma página, depois da correção do singular), 1 GET + 0 ou 1 POST
medidos, página lembrada por lista; PEN2 NOT_RUN (login com 503).

| Recusa antes do POST | Subcódigo |
| --- | --- |
| Visão diferente da resumida | `pagina_do_controle_exige_visao_resumida` |
| Charset da resposta sem codificação suportada | `charset_da_tela_do_controle_nao_suportado` |
| `hdn{T}PaginaAtual` ausente ou não decimal | `pagina_atual_da_tabela_ilegivel` |
| `hdn{T}PaginaAtual` repetido | `campos_de_pagina_atual_da_tabela_repetidos` |
| Índice incompatível com a faixa | `pagina_atual_da_tabela_incompativel_com_a_faixa` |
| N maior que o número de páginas | `pagina_fora_do_intervalo` |
| Tabela com mais de uma página sem paginador publicado | `paginador_da_tabela_nao_publicado` |
| Controle do formulário fora dos previstos | `controle_nao_previsto_no_formulario_do_controle` |
| Caixa de seleção marcada na tela | `caixa_de_selecao_marcada_no_formulario_do_controle` |
| Paginador sem opção selecionada | `paginador_sem_opcao_selecionada` |
| Paginador com mais de uma opção selecionada | `paginador_com_mais_de_uma_opcao_selecionada` |
| Opção selecionada sem `value` | `opcao_selecionada_do_paginador_sem_valor` |
| Controle previsto repetido | `controle_repetido_no_formulario_do_controle` |

| Recusa após o POST | Subcódigo |
| --- | --- |
| Rótulo da unidade diferente | `unidade_alterada_apos_paginar` |
| `hdnTipoVisualizacao` diferente | `visualizacao_alterada_apos_paginar` |
| Hash de critérios diferente | `criterios_recebidos_alterados_apos_paginar`, `criterios_gerados_alterados_apos_paginar` |
| Total diferente | `total_declarado_alterado_apos_paginar` |
| Faixa diferente da página N | `faixa_da_resposta_diferente_da_pagina_solicitada` |
| 302 | `redirecionamento_apos_paginar_controle` |
| Login, 5xx, 429, MIME, charset, tamanho | subcódigos do transporte e do envio |

Um 3xx (fora 302), 401/403, 429, 5xx ou outro status no GET ou no POST
publica também `status_http` e, quando o corpo rendeu texto,
`mensagem_do_servidor` saneada, como no
[contrato da CLI](../cli.md).
Nenhum deles autoriza repetir o POST.

Recusas de derivação do primitivo (formulário, `action`, `infra_hash`,
unidade) mantêm os subcódigos da
[submissão derivada](../../docs/arquitetura/0002-submeter-formulario-observado.md).
O `Debug` da submissão e dos erros não imprime `infra_hash`, cookie,
`Referer` nem valores.
