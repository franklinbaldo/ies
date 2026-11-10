---
type: CommandContract
title: Buscar documento pela pesquisa Web do SEI
operation: documento.buscar
command: ies documento buscar TERMO [--pagina N] [--tipo TIPO] [--assunto TEXTO] [--de DD/MM/AAAA] [--ate DD/MM/AAAA] [--json]
effect: read
implementation: partial
source_commit: db34008
---

# Busca de documento pela pesquisa Web (Refs #61, #56)

`ies documento buscar TERMO [--pagina N] [--json]` executa a mesma
pesquisa Web de [`ies processo buscar`](processo-buscar.md) (tela
`frmPesquisaProtocolo`, sessão de [`ies login`](sessao-login.md), sem
relogin, sem REST/SOAP), com a pesquisa **em documentos**. Sintaxe
aprovada pelo proprietário em 04/11/2026; o stub `documento buscar
[--assunto]` foi substituído (`--assunto` nunca foi implementado; filtros
só entram com uso e evidência).

## Entrada

`TERMO` é texto opaco digitado no campo Protocolo (número SEI exato ou
parcial). Controles (CR, LF, tab…) são recusados **antes** de aparar
espaços (`termo_de_documento_contem_controle`); vazio é
`termo_de_documento_vazio`; os dois saem com 2 (erro do clap)
**antes** de ler a configuração. O termo não é normalizado (zeros à
esquerda e pontuação vão como digitados). `--pagina N` é inteiro ≥ 1.
Os argumentos de leitura de `ies documento DOC` (`--processo`,
`--versao`, `--saida`…) junto com `buscar` são
`opcoes_incompativeis_com_busca` (saída 2). `--unidade` é pré-condição
conferida no Controle.

## Fluxo e orçamento

Igual a `processo buscar` (mesmo link do menu, mesmas telas, mesmo
primitivo de formulário e transporte), com a derivação do POST em
documentos:

- `rdoPesquisarEm` = valor de `#optDocumentos` (nome `rdoPesquisarEm`);
- `chkSinDocumentosGerados` e `chkSinDocumentosRecebidos`: os servidos
  com `checked`, com o valor servido (as duas vêm marcadas nas sete
  instalações). `onSubmitForm()` exige ao menos uma: sem nenhuma marcada
  é `NenhumTipoDeDocumentoMarcadoNaTela`, sem POST;
- ocultos como servidos (`hdnIdAssinante` servido preenchido filtraria
  em silêncio: `AssinanteOcultoPreenchidoNaPesquisaEmDocumentos`),
  `selOrgaoPesquisa[]` com as opções servidas selecionadas,
  `sbmPesquisar` e `txtProtocoloPesquisa=TERMO`.

Orçamento: página 1 = **GET + GET + POST** (3); página N = mais **um
POST** de `navegar(inicio)` (4). Sem retry, sem redirect seguido, sem
restaurar página ou filtro. Efeito: `PaginaOuFiltroLembrado`.

## Saídas (JSON, saída 0)

Todas têm `resultado`, `termo` (aparado), `unidade` (do Controle),
`origem: "web"` e `requisicoes`.

- **302** para `procedimento_trabalhar` com `id_documento` decimal
  (observado nas sete instalações para o número SEI exato), conferido
  como em `processo buscar` e **nunca seguido**:
  `{"resultado":"documento_unico","documento":{"id":"…"},"processo":{"id":"…"}}`.
  Só os ids internos: número SEI e NUP não são inventados a partir do
  termo. 302 sem `id_documento` (a forma do processo) é
  `IdDocumentoAusenteNoRedirecionamento`.
- **200 com `table.pesquisaResultado`**: `{"resultado":"lista",
  "total_declarado":N,"faixa_observada":{"inicio":I,"fim":F},
  "linhas_ausentes_da_faixa":A,"documentos":[{"numero_sei":"…",
  "rotulo":"…","processo":"…","unidade":"…","inclusao":"dd/mm/aaaa",
  "anexo":false}]}`. Barra, faixa e
  contagem de linhas com as mesmas regras de `processo buscar`, inclusive
  `linhas_ausentes_da_faixa` (A) quando a tabela tem menos registros
  que a faixa escrita na barra.
  `numero_sei` e `processo` são opacos, como servidos; `rotulo` é o
  texto do link do documento (tipo e número na árvore); `unidade` e
  `inclusao` são `null` quando a linha não os mostra; `anexo` diz se o
  link do documento é `documento_download_anexo` (e não
  `documento_visualizar`).
- **200 com `div.pesquisaSemResultado`**: `{"resultado":"sem_resultado",
  "total_declarado":0,"faixa_observada":null,"documentos":[]}`. Resposta
  observada para esta conta/unidade, **não** prova de inexistência.

## Linha de documento (forma observada)

Cada `tr.pesquisaTituloRegistro` tem `td.pesquisaTituloEsquerda` com o
`a.protocoloNormal` do processo (`procedimento_trabalhar` com
`id_procedimento` e `id_documento` decimais; `a.arvore` opcional com o
**mesmo** `href`) e um `a.protocoloNormal` do documento
(`documento_visualizar` com o mesmo `id_documento`, ou
`documento_download_anexo` com `id_anexo` decimal), e
`td.pesquisaTituloDireita` com um `a.protocoloNormal` de `href` idêntico
ao do documento, cujo texto é o número SEI. As linhas seguintes até o
próximo título trazem `td.pesquisaMetatag` "Unidade:", "Usuário:" e
"Inclusão:" (só unidade e inclusão são publicadas). Qualquer outra forma
falha fechado (subcódigos em [erros tipados](../../docs/erros-tipados.md)).

## Erros tipados (saída 1)

Os de `processo buscar` (link, tela, termo e `maxlength`, envio,
validação, 302, barra, paginação, transporte e
`BloqueioDeSegurancaDaInstalacao`), mais:

- tela: `opcao_de_documentos_ausente_na_tela`,
  `opcoes_de_documentos_repetidas_na_tela`,
  `opcao_de_documentos_sem_valor_na_tela`,
  `OpcaoDeDocumentosComNomeDiferenteDeRdoPesquisarEm`,
  `caixa_de_tipo_de_documento_ausente_na_tela`,
  `caixas_de_tipo_de_documento_repetidas_na_tela`,
  `caixa_de_tipo_de_documento_sem_valor_na_tela`,
  `NenhumTipoDeDocumentoMarcadoNaTela`,
  `AssinanteOcultoPreenchidoNaPesquisaEmDocumentos`;
- 302: `IdDocumentoAusenteNoRedirecionamento`,
  `IdDocumentoRepetidoNoRedirecionamento`,
  `IdDocumentoNaoDecimalNoRedirecionamento`;
- linha: `celula_do_titulo_do_documento_ausente_ou_repetida`,
  `LinhaDeDocumentoSemIdDocumentoNoLinkDoProcesso`,
  `link_do_documento_ausente_na_linha`,
  `links_do_documento_repetidos_na_linha`,
  `link_da_linha_de_documento_nao_reconhecido`,
  `LinkDoDocumentoDiferenteDoProcesso`,
  `NumeroSeiComLinkDiferenteDoDocumento`,
  `numero_sei_ausente_na_linha`, `rotulo_do_documento_ausente`,
  `metadado_do_documento_repetido`,
  `data_de_inclusao_fora_da_forma_observada`,
  `LinhaDaPesquisaForaDeUmRegistro`,
  `numero_sei_repetido_na_pesquisa`.

## Evidência e validação

- 26 respostas de pesquisa em documentos do acervo público (sete
  instalações) e as páginas de `pesquisa-paginas-*`; Todos
  (`documento.buscar` envia `rdoPesquisarEm=D` com o protocolo):
  [matriz](../evidencias/documento-buscar.md) e
  [decisão](../../docs/decisoes/documento-buscar.md).
- [Replay do acervo](../../docs/validacao-replay/2026-11-04-documento-buscar-acervo.md).
- [Superfície da CLI](../../tests/documento_buscar_web.rs).
- [Ensaio live na PEN3](../../docs/validacao-live/2026-11-04-documento-buscar.md),
  inclusive número SEI exato só de dígitos (302 → `documento_unico`).
- Live **PASS** na UnB 4.0.9 (pedaço de NUP e número SEI exato) e em GO
  5.0.3 (página de 9 linhas na faixa 1–10, `linhas_ausentes_da_faixa: 1`):
  [varredura UnB e GO](../../docs/validacao-live/2026-11-04-varredura-unb-go.md).
- [Varredura de 04/11/2026](../../docs/validacao-live/2026-11-04-varredura-ufrj-pen3-pen4.md):
  fragmento de NUP formatado (`lista`) e número SEI exato
  (`documento_unico`) **PASS** na UFRJ 4.1.5 e na PEN4; fragmento na PEN3.


## Filtros experimentais (#247)

As flags `--tipo TIPO`, `--assunto TEXTO`, `--de DD/MM/AAAA` e
`--ate DD/MM/AAAA` também são aceitas por `documento buscar`. O tipo
é o nome **exato** da série na lista `selSeriePesquisa` da própria tela;
período deriva de `txtDataInicio` e `txtDataFim`. A ausência de controle
é `CriterioNaoOferecidoPelaTela` sem POST; nome sem opção exata é
`TipoNaoOferecidoPelaTela`. A busca continua com 3 requisições (4 com
página maior que 1), sem retries.

`--assunto` ainda falha fechado: o autocompletar depende do
`hdnIdAssunto` resolvido, não do texto solto. Testes live: **NOT_RUN**;
a presença desses filtros varia por instalação.

### Recusas próprias dos filtros

- `CriterioNaoOferecidoPelaTela`: controle inexistente ou assunto sem ID resolvido.
- `TipoNaoOferecidoPelaTela`: lista sem a opção exata solicitada.
- `ControleDeFiltroAmbiguo`: campos ou opções repetidos.
- `AtributoDoFiltroIlegivel`: atributo HTML do filtro inválido.
- `valor_do_filtro_diferente_do_solicitado`: incompatibilidade na página seguinte (reservado à verificação de consistência).
- `data_da_pesquisa_invalida` e `periodo_da_pesquisa_invertido`: falhas de uso antes da configuração.
