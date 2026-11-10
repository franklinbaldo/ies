---
type: CommandContract
title: Buscar processo pela pesquisa Web do SEI
operation: processo.buscar
command: ies processo buscar TERMO [--pagina N] [--tipo TIPO] [--assunto TEXTO] [--de DD/MM/AAAA] [--ate DD/MM/AAAA] [--json]
effect: read
implementation: partial
source_commit: fffbc5abd07d44ccef79b3c0e6872e0bdc39c18b
---

# Busca de processo pela pesquisa Web (Refs #58, #56)

Sem relogin automático e sem REST/SOAP, `ies processo buscar TERMO
[--pagina N] [--json]` executa a pesquisa Web por protocolo com a sessão
dos arquivos privados gravados por [`ies login`](sessao-login.md)
(`IES_ORIGEM_FILE`, `IES_COOKIE_FILE`, `IES_URL_CONTROLE_FILE` ou
`IES_CONFIG_FILE`). O recorte offline `ies processo NUP
--resultado-pesquisa-file` foi aposentado em 04/11/2026 (decisão do
proprietário, [processo.obter](processo-obter.md)); respostas capturadas
passam pelos mesmos leitores no replay do acervo `pesquisa_web::acervo`
(`scripts/replay pesquisa_web::acervo`; não há `ies replay`).

O stub anterior `ies processo buscar [--assunto]` foi substituído: o
verbo continua `buscar`, agora com `TERMO` posicional obrigatório e
`--pagina N`. `--assunto` deixou de existir (nunca foi implementado).

## Entrada

`TERMO` é texto opaco (NUP inteiro ou parte do número), com as regras de
`NumeroProcesso`: controles recusados antes de aparar espaços, vazio e
`sei://` recusados, saída 2 (erro do clap) **antes** de ler a
configuração. `--pagina N` é inteiro ≥ 1. Opções de leitura de
`ies processo NUP` (`--sem-documentos`, `--andamentos-completos`…) junto com
`buscar` são `opcoes_incompativeis_com_busca` (saída 2). `--unidade` é
pré-condição conferida na tela do Controle, não rótulo ecoado.

## Fluxo e orçamento

1. **GET** do Controle (URL assinada do arquivo, origem conferida antes do
   cookie); o Controle precisa ser reconhecido em qualquer visão (R ou D).
2. Seguir **somente** o link do menu que o Controle publicou com
   `acao=protocolo_pesquisar` (relativo `controlador.php?`, exatamente um
   `infra_hash` de 64 hex, mesma instalação, mesma resposta e geração).
   Ausente, repetido, absoluto ou fora da forma é recusa tipada sem rede.
3. **GET** da tela de pesquisa (`frmPesquisaProtocolo`, unidade igual à do
   Controle).
4. **POST** derivado como na [decisão](../../docs/decisoes/processo-buscar.md):
   ocultos como servidos, `rdoPesquisarEm` = valor de `#optProcessos`,
   `selOrgaoPesquisa[]` = opções servidas selecionadas, `sbmPesquisar` com
   o valor servido e `txtProtocoloPesquisa=TERMO`.
5. Com `--pagina N` (N > 1) numa lista: **um POST** de `navegar(inicio)`
   derivado da tela de resultado (ver Paginação).

Orçamento: página 1 = **GET + GET + POST** (3); página N = **GET + GET +
POST + POST** (4). Sem retry, sem redirect seguido, sem restaurar página ou
filtro (AGENTS.md, "Não restaurar a tela"). Efeito:
`AlteraEstadoDaSessao::PaginaOuFiltroLembrado`.

## Saídas (JSON, saída 0)

Todas têm `resultado`, `termo` (como digitado, aparado), `unidade` (do
Controle), `origem: "web"` e `requisicoes`.

- **302** com `Location` para `procedimento_trabalhar` (interpretado,
  nunca seguido): `{"resultado":"processo_unico","processo":{"id":"…"}}`.
  O 302 só traz o `id_procedimento` interno; o número formatado **não** é
  inventado a partir do termo.
- **200 com `table.pesquisaResultado`**: `{"resultado":"lista",
  "total_declarado":N,"faixa_observada":{"inicio":I,"fim":F},
  "linhas_ausentes_da_faixa":A,"processos":[{"numero":"…"}]}`. Total e
  faixa vêm da barra `div.pesquisaBarraD` "Exibindo I - F de N" ou, quando a lista cabe numa
  página, "N resultados" (faixa `1 … N`, sem página 2); o singular não foi
  observado e falha fechado; o separador de milhar `.`
  só é aceito entre grupos de 3 dígitos (`1.234`), qualquer outra forma
  falha fechada. Com a faixa escrita na barra, as linhas lidas podem ser
  menos que `F - I + 1` (Goiás 5.0.3 omite registros do HTML): a lista
  publica as presentes e `linhas_ausentes_da_faixa` = `F - I + 1 -`
  lidas (0 quando completa), sem dizer que está completa. Mais linhas
  que a faixa, ou qualquer diferença com "N resultados", é
  `LinhasDaTabelaDiferentesDaFaixaDaBarra`. A paginação conta
  pela barra.
- **200 com `div.pesquisaSemResultado`** (sem tabela):
  `{"resultado":"sem_resultado","total_declarado":0,"faixa_observada":null,
  "processos":[]}`. É uma **resposta observada**, não erro e **não**
  `processo_inexistente`: significa "a pesquisa não devolveu nada para esta
  conta/unidade", sujeito a permissões e restrições de acesso.

## Erros tipados (saída 1)

- `ValidacaoRecusadaPeloSei`: 200 com `txaInfraValidacao`; carrega
  `mensagem_do_servidor` saneada.
- `RespostaDaPesquisaNaoReconhecida`: 200 sem tabela, sem aviso de
  "sem resultado" e sem validação; também tabela e aviso juntos.
- `AvisoDeSemResultadoEBarraDeContagemNaMesmaResposta`: aviso de
  "sem resultado" com barra "Exibindo I - F de N" é ambíguo e não vira
  `sem_resultado`.
- Termo (antes de qualquer POST): `TermoAcimaDoMaxlengthDaTela`
  (mais unidades UTF-16 que o `maxlength` servido em
  `txtProtocoloPesquisa`), `MaxlengthDoTermoIlegivelNaTela`,
  `campos_do_termo_repetidos_na_tela`. Sem `maxlength` publicado, a tela
  não limita o termo.
- Link: `link_da_pesquisa_ausente_no_controle`,
  `links_da_pesquisa_repetidos_no_controle`,
  `LinkDaPesquisaComEnderecoAbsoluto`,
  `link_da_pesquisa_nao_reconhecido`.
- Barra: `barra_de_contagem_da_pesquisa_ausente`,
  `barras_de_contagem_da_pesquisa_repetidas`,
  `TextoDaBarraDeContagemForaDoFormato`,
  `NumeroDaBarraDeContagemForaDoFormato`,
  `FaixaDaBarraDeContagemImpossivel`,
  `LinhasDaTabelaDiferentesDaFaixaDaBarra`.
- Paginação: `pagina_da_pesquisa_fora_do_intervalo` (antes do POST de página),
  `RegraDeNavegacaoDaPesquisaNaoObservada`,
  `PaginadorDaPesquisaNaoPublicado`,
  `link_de_paginacao_da_pesquisa_fora_da_regra`,
  `ProtocoloDaTelaDeResultadoDiferenteDoPesquisado`,
  `RedirecionamentoAoPaginarAPesquisa`,
  `PaginaDaPesquisaSemResultadoAposPaginar`,
  `TotalDaPesquisaAlteradoAposPaginar`,
  `FaixaDaPesquisaDiferenteDaPaginaSolicitada`.
- Além das recusas existentes do formulário (`regras_formulario`,
  `ErroSubmissaoFormulario`), do 302 (`ErroRedirecionamentoDaPesquisa`),
  das linhas (`ErroResultadoPesquisa`) e do transporte (`status_http`).

## Paginação (regra observada)

O resultado publica `function navegar(inicio)`, que faz
`document.getElementById('hdnInicio').value = inicio;`, chama
`onSubmitForm()` e `document.getElementById('frmPesquisaProtocolo').submit();`
(sem botão no corpo), e links `javascript:navegar('K')` com K múltiplo do
tamanho da página. O IES exige essas duas linhas literais e todo link
`navegar('K')` com K decimal, múltiplo do tamanho e menor que o total.

- Tamanho da página = `F - I + 1` da barra da página 1 quando `F < N`
  (observado: 10). Páginas = ⌈N / tamanho⌉; com `F = N`, uma página.
- `--pagina 1` não envia POST extra. N > páginas (inclusive em
  `sem_resultado`, 0 páginas, e `processo_unico`, 1 página) é
  `pagina_da_pesquisa_fora_do_intervalo` **antes** do POST de página.
- O POST de página deriva da tela de resultado (mesmo formulário, termo
  servido igual ao pesquisado), sem `sbmPesquisar`, com
  `hdnInicio = (N - 1) × tamanho`. A resposta precisa ser lista com o
  mesmo total e faixa `inicio = hdnInicio + 1`,
  `fim = min(hdnInicio + tamanho, N)`.

## Links das linhas do resultado

Em cada página, cada linha da tabela tem um `a.protocoloNormal` com
`procedimento_trabalhar`; a UnB 4.0.9 publica também um `a.arvore` com o
**mesmo** `href` e repete o atributo `class` idêntico no
`a.protocoloNormal`. O processo já visitado pela sessão vem, na PEN3 e na
PEN4 5.0.4, com `class="protocoloNormal processoVisitado" …
class="protocoloNormal"` (visto ao vivo na PEN4 em 04/11/2026, que antes
saía `html_de_pesquisa_nao_interpretado`). O `class` repetido é aceito
quando **todas** as ocorrências contêm a classe procurada; `href`
diferente, mais de um `a.arvore` por linha ou uma ocorrência sem a classe
falham fechado.

Em Goiás, a linha cujo acerto é um **documento** do processo leva
`id_documento` no link do processo e mais dois `a.protocoloNormal`
(nome e número SEI do documento) que abrem `documento_visualizar`. Esses
links são aceitos só com uma `acao=documento_visualizar` e o **mesmo**
`id_documento` decimal do link do processo; senão,
`LinkDoDocumentoDaLinhaDiferenteDoProcesso`. A linha publica só o
número do processo; o documento não sai.

## Evidência e validação

- [Matriz Todos](../evidencias/processo-buscar.md) e ensaio estrutural da
  UnB 4.0.9 de 03/11/2026 (capturas privadas, só estrutura).
- [Parser e fluxo](../../src/pesquisa_web.rs), com testes de sequência de
  requisições por transporte injetável e
  [superfície da CLI](../../tests/processo_buscar_web.rs).
- [Replay do acervo](../../docs/validacao-replay/2026-11-04-processo-buscar-acervo.md)
  (04/11/2026): Controle, tela, envio e plano da página 2 das 52
  cassettes com a Pesquisa, nas sete instalações, pelas peças do comando.
- Ensaio live do comando IES: PASS na UnB e em GO
  ([03/11/2026](../../docs/validacao-live/2026-11-03-processo-buscar.md))
  e na PEN3 ([04/11/2026](../../docs/validacao-live/2026-11-04-processo-buscar.md));
  PEN1 responde `sem_resultado` até para NUP da unidade; PEN2: **NOT_RUN**.
- [Varredura de 04/11/2026](../../docs/validacao-live/2026-11-04-varredura-ufrj-pen3-pen4.md):
  UFRJ 4.1.5 PASS (dentro e fora da caixa, lista, `--pagina 2`; NUP
  gerado no dia sai `sem_resultado`); PEN4 PASS em NUP dentro e fora da
  caixa e `sem_resultado`, lista **FAIL → corrigido offline** (processo
  visitado), lista ao vivo depois da correção NOT_RUN.


## Filtros experimentais (#247)

`ies processo buscar TERMO [--pagina N] [--tipo TIPO] [--assunto TEXTO]
[--de DD/MM/AAAA] [--ate DD/MM/AAAA]` preserva o termo e o fluxo
Web anterior. `--tipo` procura o texto **exato** de uma opção de
`selTipoProcedimentoPesquisa`, usando o seu `value` servido no mesmo
POST. `--de` e `--ate` usam `txtDataInicio` e `txtDataFim`, somente
quando presentes na tela. Datas inválidas e período invertido são entrada
inválida antes da rede. Os filtros não acrescentam GET nem POST (3
requisições na primeira página; 4 na página N).

A opção `--assunto` existe na borda, mas **recusa por enquanto** com
`CriterioNaoOferecidoPelaTela`: `txtAssunto` e `hdnIdAssunto`
formam um autocompletar dependente de identificador, e enviar apenas texto
não filtra. Não há consulta adicional autorizada para resolver esse ID.
Também se recusa qualquer campo ausente da tela; tipo sem opção exata é
`TipoNaoOferecidoPelaTela`; controles repetidos ou valor diferente
na página seguinte são recusados. O ensaio live UnB/PEN3 para os novos
filtros está **NOT_RUN**; não inferir suporte universal.

### Recusas próprias dos filtros

- `CriterioNaoOferecidoPelaTela`: controle inexistente ou assunto sem ID resolvido.
- `TipoNaoOferecidoPelaTela`: lista sem a opção exata solicitada.
- `ControleDeFiltroAmbiguo`: campos ou opções repetidos.
- `AtributoDoFiltroIlegivel`: atributo HTML do filtro inválido.
- `valor_do_filtro_diferente_do_solicitado`: incompatibilidade na página seguinte (reservado à verificação de consistência).
- `data_da_pesquisa_invalida` e `periodo_da_pesquisa_invertido`: falhas de uso antes da configuração.
