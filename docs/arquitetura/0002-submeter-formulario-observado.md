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
tipadas, mais uma função de envio injetável (`_via` + `FnOnce`).
`ies processo buscar` continua como descrito em
[seu contrato](../../contracts/cli/processo-buscar.md). O primeiro uso
live é `ies unidade recebidos|gerados --pagina N` (ensaiado em 02/11/2026
com a sintaxe anterior `listar-processos --categoria T --pagina N`)
([páginas da caixa](../../contracts/cli/unidade-listar-processos-paginas.md)),
com a `const` `OBTER_PAGINA_DO_CONTROLE` em `src/pagina_do_controle.rs`.

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
  ordem do documento, salvo os listados pela operação em
  `campos_ocultos_reescritos` (o que o JavaScript observado dela grava,
  como `hdn{T}PaginaAtual` da caixa): o valor informado ocupa a posição
  servida, e oculto fora da lista ou não publicado é recusado. Campos
  ocultos exigidos pela operação ausentes,
  ou nomes ocultos repetidos, são recusados. Ocultos fora do `<form>`
  (por exemplo `hdnInfraPrefixoCookie`) não participam, nem controles
  `disabled`, que o navegador também omite.
- Campos do usuário só entram se estiverem na lista da operação **e**
  publicados pelo formulário; nome fora da lista, não publicado ou
  informado duas vezes tem recusa própria. A única exceção à repetição
  é um `<select multiple>` servido: o navegador envia um par por opção
  selecionada, e os pares saem juntos na posição do `select`. O
  primitivo não escolhe as opções; quem as lê da tela é a operação.
- `EnvioDoFormulario::BotaoDeEnvio(nome)`: o botão efetivo (`type=submit`
  com o nome declarado) participa com nome e valor servidos; sem ele, recusa.
  `EnvioDoFormulario::SubmitPorScript`: a tela chama `submit()` por
  JavaScript (paginador da caixa, #57) e nenhum botão participa.
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
Não há evidência de ausência de efeito, então a leitura conservadora vale.
Além disso, a resposta 302 observada (abaixo) aponta para
`procedimento_trabalhar`, cuja abertura pode registrar recebimento.

## Invariantes

- Sem redirects: nenhum 3xx é seguido. Após o envio, só o **302** volta
  como dado (`RespostaDoEnvio::Redirecionamento`), com o `Location` cru
  numa estrutura de `Debug` redigido e sem ler o corpo; os outros 3xx
  continuam `Transporte(Redirecionamento)`. A interpretação é da operação
  (seção abaixo). O GET do Controle não muda: 3xx segue como erro.
  401/403 é `Transporte(Status(StatusHttpRecusado { status: 401 | 403, .. }))`.
- Limite de 8 MB durante o recebimento (reuso de `decodificar_resposta`);
  tela de entrada acima do limite também é recusada.
- Envio por `FnOnce`: a submissão é consumida, sem retry após erro ou
  timeout, e sem troca de backend.
- Tela de login com HTTP 200 após o envio é recusa própria.
- HTTP 200 sem o marcador de resultado declarado pela operação
  (`table.pesquisaResultado`) é recusa: o IES não sabe qual marcação
  o SEI usa para recusas de pesquisa e não inventa uma.
- Arquivos de cookie seguem `0600`, sem symlink.

## Redirecionamento 302 da pesquisa exata

Nas quatro cassettes inspecionadas (UnB, PEN1, UFRJ e Goiás, interações
POST 6 e 10 de `processo-busca-*`), o 302 tem corpo vazio e cabeçalho
`location` **relativo**, sem fragmento e com `&` literal:
`controlador.php?acao=procedimento_trabalhar&acao_origem=protocolo_pesquisar&id_procedimento=<decimal>&id_documento=&infra_sistema=<decimal>&infra_unidade_atual=<decimal>&infra_hash=<64 hex>`.
Nas oito respostas, `infra_unidade_atual` do `Location` é igual ao da URL
do POST. O Todos seguiu o `Location` (GET `procedimento_trabalhar` na
interação seguinte); o IES **não segue**. As cassettes não gravam o corpo
do POST: que o 302 corresponda a protocolo exato com um único processo
é a intenção do cenário, não fato comprovado. Os POSTs 2 e 4, com 200,
traziam tabela de resultados com vários links de processo.

`ResultadoPesquisaProtocolo` (código da operação, em
`src/pesquisa_web/formulario_pesquisa.rs`) tem duas formas:
`Tabela(Vec<ResultadoPesquisa>)` e
`RedirecionadoParaProcessoUnico { id: IdProcedimento }`. O `Location`
(regras em `src/pesquisa_web/redirecionamento.rs`) precisa, nesta ordem:
existir com até 8 KiB; resolver contra a URL do POST para uma URL
HTTPS sem fragmento; pertencer à mesma `Instalacao`
(`autoriza_controlador`); ter nomes de parâmetro canônicos e nenhum `;`;
um único `acao=procedimento_trabalhar`; um único
`acao_origem=protocolo_pesquisar`; um único `id_procedimento`
decimal; um único `infra_hash` de 64 hexadecimais; um único
`infra_unidade_atual` igual ao da URL do POST; e um único
`infra_sistema` decimal igual, sem normalizar zeros, ao da URL do POST.
A URL do POST é o `action` assinado do formulário; sem `infra_sistema`
nela, a interpretação falha fechada (`InfraSistemaAusenteNaUrlDoEnvio`).
Cada falha tem erro próprio em `ErroRedirecionamentoDaPesquisa`.

Ensaio offline: os oito `Location` reais (UnB, PEN1, UFRJ, Goiás;
interações 6 e 10), com a URL do POST da mesma resposta, passaram pela
interpretação e devolveram `id_procedimento` decimal (**PASS**, teste
local temporário lendo arquivo `0600` fora do repositório, não versionado).
As exigências de `acao_origem` e `infra_sistema` (#58) vieram depois desse
ensaio: o `Location` observado traz ambos, mas a igualdade de
`infra_sistema` com a URL do POST real **não foi reensaiada**
(**NOT_RUN** nesta revisão, sem cassettes locais).
Os testes versionados usam só valores sintéticos.

Abrir `procedimento_trabalhar` pode registrar recebimento
([efeitos](../como-o-sei-funciona/efeitos-e-verificacao.md)): o
redirecionamento é **interpretado, não seguido**. A URL assinada é
descartada depois de extrair o identificador, que é interno e não entra
na saída da CLI. Sessão vencida que redirecione para o login cai em
`LocalizacaoDoRedirecionamentoForaDaInstalacao` ou em ação diferente.

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

**Mudanças no primitivo nesta revisão, e por que não ficam na operação:**
(1) o envio devolve o 302 como dado. O status e o `Location` só existem
dentro de `enviar_submissao_via`, que antes os convertia em erro; a
operação não teria como interpretá-los sem o primitivo entregá-los. O
primitivo não interpreta nem segue, e a `OperacaoDeFormulario` não ganhou
campo. (2) a repetição do nome de um `<select multiple>` é semântica do
HTML, igual para qualquer formulário; a regra de quais opções enviar
(`selected` servido do órgão) e as regras do `onsubmit` ficam na
[decisão da operação](../decisoes/processo-buscar.md). Duas funções de
leitura (`localizar_formulario`, `decodificar_atributo`) passam a
`pub(crate)` para a operação reler a mesma tela, sem novo parâmetro.

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
  As regras escritas à mão a partir desse código estão na
  [decisão da operação](../decisoes/processo-buscar.md). A derivação
  **não** executa JavaScript nem serializa outros controles visíveis
  (selects, checkboxes marcados): diferença conhecida em relação ao
  navegador, a ser medida por ablação em treinamento.
- Nas duas instalações, dois POSTs responderam 200 com
  `table.pesquisaResultado` e dois responderam **302** para
  `procedimento_trabalhar` (interações 6 e 10), interpretado sem seguir
  conforme a seção acima.
- As cassettes públicas (`schema_version: 1`) registram para POST só
  `method`, `operation`, `operation_parameter` e `pathname`: **não há
  corpo nem cabeçalhos** da requisição (verificado em
  `processo-busca-unb.json` e `processo-busca-pen1.json`). Os testes provam coerência interna com o
  formulário servido, **não** que o SEI aceita o payload.
- A paginação POST do Controle de Processos nunca foi capturada.
- Resposta de pesquisa sem resultados não foi observada; o marcador
  pode faltar nesse caso e a recusa seria falso-negativo.
- O `tl` 0.7.8 perde o primeiro caractere do atributo que segue um
  booleano sem valor (`multiple disabled` é lido como `isabled`) e monta
  com essa chave os índices de `id`/classe e o `inner_html`.
  **Corrigido** em `src/html_atributos.rs`: tokenizador próprio da tag
  de abertura (`HTMLTag::raw()`), com nomes em minúsculas, valores
  brutos, atributo repetido valendo o primeiro, como no navegador (#676), e tag
  ilegível como `AtributoHtml(_)`. O `tl` segue dono da árvore. Testes:
  `formulario_web/tests_atributos.rs`, regressões por leitor (booleano
  `itemscope` antes de cada atributo) e a guarda
  `tests/atributos_html_pelo_tokenizador.rs`.
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
