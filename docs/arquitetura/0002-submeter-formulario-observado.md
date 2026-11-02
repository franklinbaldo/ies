---
type: ArchitectureDecision
title: Submissão derivada de formulário Web observado
description: Primitivo offline que deriva um POST do formulário servido na mesma sessão, com recusas tipadas
status: proposed
decision: derive-from-served-form per-operation const, offline only
---

# 0002 — Submeter formulário Web observado

## Problema

Operações Web como a pesquisa de protocolo (`protocolo_pesquisar`,
issue #58) exigem um POST montado a partir de um formulário HTML.
O [conhecimento sobre formulários](../como-o-sei-funciona/formularios.md)
mostra que o `action` publicado é assinado, que campos ocultos carregam
estado opaco e que o JavaScript da tela pode reescrever o payload.
Montar o POST "de cabeça" fabricaria estado e assinatura; recusar sempre
impede qualquer fluxo Web fora da caixa.

Este documento define **apenas a derivação** da submissão e suas recusas
tipadas, mais uma função de envio injetável (`_via` + `FnOnce`). Nenhum
comando da CLI usa o primitivo nesta fatia: `ies processo buscar` continua
como descrito em [seu contrato](../../contracts/cli/processo-buscar.md),
nenhum caminho live é habilitado e a matriz de capacidades não muda.

Aqui, `action` é sempre o **atributo HTML `action` do `<form>`**, não o
antigo despachante `Action` removido do IES.

## Fonte da requisição

- O formulário vem somente do HTML servido na **mesma geração de sessão**,
  junto com a URL exata que o serviu (`TelaServida`). Nunca de cache,
  de outra sessão ou de outra unidade.
- O formulário é localizado pelo `id` ou `name` declarado na operação.
  Zero ocorrências ou mais de uma são recusas distintas; o primeiro
  encontrado nunca é escolhido.
- A URL servida precisa pertencer à `Instalacao` (host e caminho-base),
  e a unidade publicada em `lnkInfraUnidade` precisa ser a solicitada,
  antes de qualquer derivação.

## Derivação

- Campos `hidden` do formulário entram **exatamente como servidos**, na
  ordem do documento; campos ocultos exigidos pela operação ausentes,
  ou nomes ocultos repetidos, são recusados. Ocultos fora do `<form>`
  (por exemplo `hdnInfraPrefixoCookie`) não participam.
- Campos do usuário só entram se estiverem na lista da operação **e**
  publicados pelo formulário; nome fora da lista, não publicado ou
  informado duas vezes tem recusa própria.
- O botão de envio efetivo (`type=submit` com o nome declarado) participa
  com nome e valor servidos; sem ele, recusa.
- `hdnAcao` só seria preenchido por uma regra observada por operação.
  Nenhuma operação tem regra nesta fatia: se o formulário publicar
  `hdnAcao`, a derivação recusa em vez de inferir o valor.
- O `action` é resolvido contra a **URL servida** (não a raiz da
  instalação), descartando o fragmento (`#ancoraBarraPesquisa`, observado).
  O resultado passa por `Instalacao::autoriza_controlador`, precisa ter
  um único `acao` igual ao da operação e um único `infra_hash` de 64
  hexadecimais. Ausente, fora da instalação, sem hash ou de outra ação
  são recusas distintas.

## Codificação e cabeçalhos

- Somente `application/x-www-form-urlencoded`: `enctype` diferente e
  `accept-charset` publicado são recusados (não observados).
- O charset é o da resposta que serviu a tela. UnB e PEN1 serviram
  `text/html; charset=iso-8859-1`; valores são codificados em
  ISO-8859-1 (`ação` vira `a%E7%E3o`). Caractere não representável é
  recusado, sem a substituição numérica que navegadores fazem.
- `Origin` é `https://host` e `Referer` é a URL servida. O cookie vem
  de arquivo privado `0600`, lido só depois da derivação validar a
  instalação, e nunca entra na submissão derivada.
- `Debug` da submissão e dos erros não imprime `infra_hash`, cookie,
  `Referer` ou valores de campos.

## Avaliação de efeito

Cada operação declara `EfeitoAvaliado`, sem valor padrão:
`ConsultaSemEfeitoObservado { evidencia }`, `AlteraEstadoDaSessao { o_que }`
ou `Desconhecido`. `Desconhecido` recusa a derivação.

Para `protocolo_pesquisar` a classificação é
`AlteraEstadoDaSessao::PaginaOuFiltroLembrado`. A
[nota de paginação](../como-o-sei-funciona/tabelas-e-paginacao.md) registra
que o servidor pode lembrar página e filtro da sessão, e a
[nota de efeitos](../como-o-sei-funciona/efeitos-e-verificacao.md) só
observou como consulta a `protocolo_pesquisa_rapida`, que é outra ação.
Não há evidência de ausência de efeito, então a variante conservadora vale.
Além disso, a resposta 302 observada (abaixo) aponta para
`procedimento_trabalhar`, cuja abertura pode registrar recebimento.

## Invariantes

- Sem redirects: 3xx após o envio é `Transporte(Redirecionamento)`;
  401/403 é `Transporte(AcessoNegado)`.
- Limite de 8 MB durante o recebimento (reuso de `decodificar_resposta`);
  tela de entrada acima do limite também é recusada.
- Envio por `FnOnce`: a submissão é consumida, sem retry após erro ou
  timeout, e sem troca de backend.
- Tela de login com HTTP 200 após o envio é recusa própria.
- HTTP 200 sem o marcador de resultado declarado pela operação
  (`table.pesquisaResultado`) é recusa: o IES não sabe qual marcação
  o SEI usa para recusas de pesquisa e não inventa uma.
- Arquivos de cookie seguem `0600`, sem symlink.

## Fronteira com o despachante proibido

A [decisão 0001](0001-rotas-por-evidencia.md) proíbe `Action`, `agir` e
`PageModel` genéricos.

**Permitido:** uma função `pub(crate)` de derivação/validação que recebe uma
`OperacaoDeFormulario` `const` declarada pela própria operação, e a função
de envio `_via`. A `const` de `protocolo_pesquisar` mora junto do código
de pesquisa (`src/pesquisa_web/formulario_pesquisa.rs`).

**Proibido:** trait de operação; registro ou lista global de operações;
descoberta de formulários na tela; mapa genérico de campos vindo de fora;
`enum` que enumere "todas as ações"; qualquer item público.

**Desempate:** se uma segunda operação precisar de caso especial, duplicar
o tratamento na operação em vez de parametrizar o primitivo.

## Evidências e lacunas

Formulários inspecionados offline (só nomes e estrutura, sem valores):
UnB, `processo-busca-unb.json` do ZIP público (SHA-1
`21e0559015c56c403daf8933e5040504a7b83837`), e PEN1,
`processo-busca-pen1.json` do lote `7e4954690958f7a7` (mesmo SHA-1 do
[card](../../cassettes/pen1/processo-busca-pen1--c06d64484496.md)).

- Formulário `frmPesquisaProtocolo` (id e name), `method=post`,
  `onsubmit="return onSubmitForm();"`, `action` relativo
  `controlador.php?acao=protocolo_pesquisar&acao_origem=protocolo_pesquisar&…&infra_hash=…#ancoraBarraPesquisa`.
- Ocultos: `hdnInfraTipoPagina`, `hdnIdUnidade`, `hdnIdAssunto`,
  `hdnIdAssinante`, `hdnIdContato`, `hdnIdUsuarioGerador1..3`, `hdnInicio`.
  Botão `sbmPesquisar` (`type=submit`, valor `Pesquisar`). Campo do
  protocolo `txtProtocoloPesquisa`; rádio `rdoPesquisarEm` (`P`/`D`).
- Sem `hdnAcao` e sem `enctype`. A lista de controles foi **idêntica** na
  UnB e na PEN1; a única diferença foi o comprimento do valor de
  `hdnInfraPrefixoCookie`, que fica fora do formulário. A tela de
  resultado acrescenta `sbmPesquisarSalvar` e mais um botão.
- `navegar(inicio)` reescreve `hdnInicio` via JavaScript (paginação);
  `onSubmitForm()` exige rádio marcado e órgão em `selOrgaoPesquisa[]`.
  A derivação **não** executa JavaScript nem serializa controles visíveis
  fora da lista (selects, checkboxes marcados): diferença conhecida em
  relação ao navegador, a ser medida por ablação em treinamento.
- Nas duas instalações, dois POSTs responderam 200 com
  `table.pesquisaResultado` e dois responderam **302** para
  `procedimento_trabalhar` (interações 6 e 10). Esta fatia recusa o 302;
  tratar o `Location` exige decisão própria, sem segui-lo.
- As cassettes públicas (`schema_version: 1`) registram para POST só
  `method`, `operation`, `operation_parameter` e `pathname`: **não há
  corpo nem cabeçalhos** da requisição (verificado em
  `processo-busca-unb.json`). Os testes provam coerência interna com o
  formulário servido, **não** que o SEI aceita o payload.
- A paginação POST do Controle de Processos nunca foi capturada.
- Resposta de pesquisa sem resultados não foi observada; o marcador
  pode faltar nesse caso e a recusa seria falso-negativo.
- Validação live: **NOT_RUN** (sem instalação autorizada nesta fatia).

## Alternativas

1. **Somente replay offline** (situação atual de `processo.buscar`):
   zero requisições, mas nenhuma busca fresca.
2. **POST fixo por operação**, com campos e URL montados no código:
   mais simples, porém fabrica estado oculto e assinatura, contrariando
   a [nota de `infra_hash`](../como-o-sei-funciona/infra-hash.md).
3. **REST WSSEI ou SOAP**: módulo e credenciais não comprovados por
   instalação; prioridade baixa conforme as instruções do projeto.
4. **Derivar do formulário servido** (escolhida): reproduz o estado
   publicado e falha fechado, ao custo de uma leitura GET prévia e de
   não reproduzir o JavaScript da tela.

## Condições de revisão

- Captura autorizada do corpo do POST em treinamento mostrar campos
  que a derivação omite ou serializa de outro modo.
- Ablação demonstrar que algum oculto é dispensável, ou que um controle
  visível é obrigatório.
- Instalação com `enctype`, `accept-charset`, charset UTF-8 ou `hdnAcao`
  diferentes do observado.
- Observação da marcação de recusa ou de pesquisa vazia.
- Segunda operação precisar do primitivo: aplicar o desempate acima.
