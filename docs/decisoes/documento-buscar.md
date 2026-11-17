---
type: ImplementationDecision
title: Pesquisa de documento pela Web
operation: documento.buscar
implementation: partial
status: adopted-for-scope
scope: pesquisa em documentos pelo campo Protocolo (número SEI exato ou parcial) na tela de Pesquisa, pela sessão de ies login, página observada ou --pagina N
selected_strategy: web-post-authorized
alternatives: web-post-authorized,web-captured-offline,rest-wssei,soap-official
evidence_level: replay-do-acervo-sete-instalacoes-e-ensaio-live-pen3
live_validation: pass-pen3-termo-numerico-not-run
---

# Decisão de implementação de documento.buscar

Sintaxe aprovada pelo proprietário em 04/11/2026 (issue #61):
`ies documento buscar TERMO [--pagina N]`. Antes, `documento buscar
[--assunto]` era stub; depois, `TERMO` posicional e `--pagina N`, como em
`processo buscar`. Filtros (assunto, tipo, período) só com uso e evidência.

## Alternativas consideradas

- **web-post-authorized:** a mesma tela, o mesmo link do menu, o mesmo
  primitivo de formulário e o mesmo transporte de
  [`processo buscar`](processo-buscar.md), com o rádio em documentos.
  Orçamento: 3 requisições na página 1 e 4 na página N.
- **web-captured-offline:** ler respostas já capturadas (como o antigo
  `ies replay --documento --termo-documento`, retirado em 05/11/2026:
  [replay fora da CLI](acervo/replay-fora-da-cli.md)); não responde pela conta
  atual nem acha documento fora das capturas.
- **rest-wssei:** módulo opcional e token não comprovados por instalação.
- **soap-official:** autenticação e equivalência desconhecidas.

## Comparação

A rota Web reaproveita o fluxo validado ao vivo de `processo buscar`
(UnB, GO, PEN3) e só muda a derivação e o leitor. O Todos
(`documento.buscar`) envia a mesma tela com `rdoPesquisarEm=D`, as duas
caixas de tipo e o protocolo; com o número SEI exato, as sete cassettes
`documento-numero-e-busca-*` gravaram 302 para `procedimento_trabalhar`
com `id_documento` decimal (a busca de processo grava `id_documento=`
vazio). REST e SOAP continuam sem disponibilidade comprovada.

## Decisão

- Um primitivo: `derivar_pesquisa_em(PesquisarEm, …)` e
  `derivar_pagina_em` (`formulario_pesquisa`) servem às duas operações;
  as regras do `onsubmit` (`regras_formulario`) leem `#optProcessos` ou
  `#optDocumentos` e, em documentos, `chkSinDocumentosGerados` e
  `chkSinDocumentosRecebidos` servidas marcadas, com o valor servido
  (`onSubmitForm()` exige uma; nenhuma é recusa sem POST). O
  `hdnIdAssinante` servido preenchido filtraria em silêncio: recusa.
- Um fluxo: `fluxo::buscar_colhendo::<A>` com `A: AlvoDaPesquisa`
  (`EmProcessos`, `EmDocumentos`), que dá o rádio, o leitor do 200, a
  forma do 302 e a contagem para a paginação (`Contagem`). Não é
  despachante: são as duas pesquisas da mesma tela.
- Uma barra: `classificar_resposta` (resposta.rs) lê barra, aviso e
  contagem de linhas para as duas; o leitor de linhas é o da operação.
- 302: conferido por `interpretar_redirecionamento` (o mesmo de
  `processo buscar`) e, depois, `id_documento` exatamente um, decimal e
  não vazio. Publica só os ids; **não é seguido**.
- Linha de documento (`documentos/linhas.rs`): processo com
  `id_documento`, documento (`documento_visualizar` com o mesmo id ou
  `documento_download_anexo` com `id_anexo`), número SEI à direita com o
  mesmo `href`, metadados "Unidade:" e "Inclusão:"; o usuário e o trecho
  não saem. Tudo que sobra na tabela é recusa.

Duas formas novas, observadas e aceitas só como servidas:

- **Processo visitado (PEN3 5.0.4).** O link do processo vem com
  `class="protocoloNormal processoVisitado"` e de novo
  `class="protocoloNormal"`. Atributo repetido na tag vale o primeiro, como
  no navegador (#676): o link é lido pelo primeiro `class`, em `documento
  buscar` e em `processo buscar`.
- **Última página com "N resultados" (PEN3 5.0.4,
  `pesquisa-paginas-pen3`, passo 9).** Na página 2 de 16 a barra diz "16
  resultados". No POST de página, essa barra vale a faixa esperada só se
  ela termina em N e o único `div.pesquisaPaginaSelecionada` é a página
  pedida; senão continua `1 … N` e a contagem de linhas recusa. Vale para
  as duas operações (mesma barra).

## Evidências verificáveis

- [Contrato](../../contracts/cli/documento-buscar.md) e
  [matriz Todos](../../contracts/evidencias/documento-buscar.md)
- [Replay do acervo](../validacao-replay/2026-11-04-documento-buscar-acervo.md)
  (52 cassettes, sete instalações) pelo [leitor do replay](../../src/pesquisa_web/acervo/leitores/documentos.rs)
- [Superfície da CLI](../../tests/documento_buscar_web.rs); testes
  unitários de [linhas, 302 e resposta](../../src/pesquisa_web/documentos/tests.rs),
  [fluxo e orçamento](../../src/pesquisa_web/documentos/tests_fluxo.rs) e
  [JSON](../../src/documento_buscar_cli/tests.rs)
- [Ensaio live na PEN3](../validacao-live/2026-11-04-documento-buscar.md)

RED registrado: 21 testes falhando (linhas, resposta, 302, fluxo e JSON)
antes da implementação; depois, 2 (processo visitado e última página com
"N resultados") antes da segunda correção.

## Linhas ausentes da faixa

A mesma regra de
[processo buscar](processo-buscar.md#linhas-ausentes-da-faixa), na peça
comum (`classificar_resposta`), por decisão do proprietário de
04/11/2026. As quatro respostas de Goiás 5.0.3 antes recusadas
(`documento-numero-e-busca-go#13`, 4 de "1 - 10"; `pesquisa-paginas-go#4`,
1 de "11 - 20"; `#5`, 6 de "21 - 30"; `#8`, tabela vazia em "1 - 10 de
3.300") têm `tr` = 3 × registros e nenhum link fora deles: as linhas não
estão no HTML. A lista publica os documentos presentes, a faixa da barra
em `faixa_observada`, `total_declarado` e `linhas_ausentes_da_faixa`.
Mais linhas que a faixa, ou diferença sob "N resultados", continuam
`linhas_da_tabela_diferentes_da_faixa_da_barra`.

RED da regra: 6 testes falhando (4 nas peças comuns e de documentos, 2 na
borda das duas CLIs) antes da implementação.

## Limitações e revisão

- **Goiás publica menos linhas que a faixa** ("Exibindo 1 - 10 de 3.300"
  com 0 a 6 linhas): publicadas com `linhas_ausentes_da_faixa`, ver
  abaixo.
- A resposta de "Erro realizando pesquisa" da UnB (`divInfraMensagens`,
  sem `txaInfraValidacao`) continua `resposta_da_pesquisa_nao_reconhecida`.
- Nenhuma cassette gravou pesquisa em documentos por número **parcial**:
  a lista vem de buscas por assunto e tipo, com as mesmas linhas; o
  parcial é o que o ensaio live observa.
- UnB, UFRJ, GO, PEN1, PEN2 e PEN4 ao vivo: **NOT_RUN** nesta rodada.
