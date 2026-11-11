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
  intervalo e linhas iguais ao total (`3 registros:`, ou `1 registro:` e
  `(1 registro):` no singular, este na UFRJ), a faixa é `1..total`.
  Tabela com zero registros não publica a chave `faixa_observada`.
- **Lista vazia sem tabela** (GO 5.0.3, unidade sem processos, replay de
  04/11/2026): o SEI omite `tbl{T}`. A lista é publicada com
  `total_declarado: 0`, sem a chave `faixa_observada` e nenhum processo **só**
  com a prova estrutural inteira: visão R e `lnkInfraUnidade`
  identificados; `div{T}AreaTabela`, `div{T}AreaPaginacaoSuperior` e
  `div{T}AreaPaginacaoInferior` únicas e só com espaços; `hdn{T}NroItens`
  e `hdn{T}PaginaAtual` únicos e com valor `0`. Faltando, repetida ou
  diferente qualquer parte, `tabela_ausente`. `--pagina 2` é
  `pagina_da_caixa_fora_do_intervalo` (`paginas: 1`). A D vazia não foi observada.
- No replay de páginas capturadas, a faixa é a união contígua das
  páginas recebidas, em qualquer ponto da lista.
- A faixa pode não começar em 1: o servidor lembra a página pedida na
  sessão. A saída não diz "primeira página".
- `paginas_do_controle_nao_obtidas` foi aposentado por #57: a página
  observada é a resposta padrão, sem obrigação de colher todas.

Continuam recusas tipadas, antes de publicar ou exportar:

| Fato observado | Erro (stderr) |
| --- | --- |
| Legenda sem `registros` (total não declarado) | `TotalNaoDeclaradoNaLegenda` |
| Legenda sem intervalo e total diferente das linhas | `LegendaSemFaixaComTotalDiferenteDasLinhas` |
| Paginador da tabela publicado e legenda sem intervalo | `PaginadorDaTabelaSemFaixaNaLegenda` |
| Faixa inválida, alterada ou páginas não contíguas | `FaixaInvalida`, `FaixaAlterada`, `FaixaDescontinua` |
| Filtro de marcador ativo (`hdnIdMarcador{unidade}` com valor; a lista é só a do marcador) | `FiltroDeMarcadorAtivoNoControle` |

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
3. N fora de `1..=páginas` → `pagina_da_caixa_fora_do_intervalo`, **sem POST**.
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
medidos, página lembrada por lista; GO 5.0.3 PASS (página de 10) na
terceira tentativa e PEN2 NOT_RUN ([GO e PEN2](../../docs/validacao-live/2026-11-04-unidade-paginacao-go-pen2.md)).

| Recusa antes do POST | Erro (stderr) |
| --- | --- |
| Visão diferente da resumida | `ErroPaginaDoControle::PaginaExigeVisaoResumida` |
| Charset da resposta sem codificação suportada | `ErroPaginaDoControle::CharsetDaTelaNaoSuportado` |
| `hdn{T}PaginaAtual` ausente ou não decimal | `PaginaAtualDaTabelaIlegivel` |
| `hdn{T}PaginaAtual` repetido | `CamposDePaginaAtualDaTabelaRepetidos` |
| Índice incompatível com a faixa | `PaginaAtualDaTabelaIncompativelComAFaixa` |
| N maior que o número de páginas | `ErroPaginaDoControle::PaginaForaDoIntervalo` |
| Tabela com mais de uma página sem paginador publicado | `PaginadorDaTabelaNaoPublicado` |
| Controle do formulário fora dos previstos | `ControleNaoPrevistoNoFormularioDoControle` |
| Caixa de seleção marcada na tela | `CaixaDeSelecaoMarcadaNoFormularioDoControle` |
| Paginador sem opção selecionada | `PaginadorSemOpcaoSelecionada` |
| Paginador com mais de uma opção selecionada | `PaginadorComMaisDeUmaOpcaoSelecionada` |
| Opção selecionada sem `value` | `OpcaoSelecionadaDoPaginadorSemValor` |
| Controle previsto repetido | `ControleRepetidoNoFormularioDoControle` |

| Recusa após o POST | Erro (stderr) |
| --- | --- |
| Rótulo da unidade diferente | `UnidadeAlteradaAposPaginar` |
| `hdnTipoVisualizacao` diferente | `VisualizacaoAlteradaAposPaginar` |
| Hash de critérios diferente | `ErroPaginaDoControle::CriteriosRecebidosAlteradosAposPaginar`, `CriteriosGeradosAlteradosAposPaginar` |
| Total diferente | `TotalDeclaradoAlteradoAposPaginar` |
| Faixa diferente da página N | `ErroPaginaDoControle::FaixaDaRespostaDiferenteDaPaginaSolicitada` |
| 302 | `ErroPaginaDoControle::RedirecionamentoAposPaginar` |
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
