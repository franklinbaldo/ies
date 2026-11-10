---
type: ArchitectureDecision
title: Erros por tipo, origem única e propagação direta
description: Tipos próximos à detecção preservam a identidade da falha; Serde apresenta o tipo sem subcódigos paralelos
status: accepted
decision: typed-errors single-origin direct-propagation serde-at-boundary
---

# 0005 — Erros por tipo

## Problema

Cada `enum` de erro mantinha uma `fn subcodigo()` manual: 966 braços de
`match` que copiavam a classificação tipada para texto. O compilador
garante que o `match` cobre as variantes, mas não que o texto corresponda
à variante. Por isso existiam tabelas de subcódigos longe do código e um
hook só para mantê-las em sincronia. Nenhum consumidor programático decide
pelo `subcodigo`: os únicos usos eram testes e o próprio verificador.

## Decisão

- **O tipo é a verdade.** Declare o erro perto do ponto que detecta a
  falha. Em regra, cada variante tem um único ponto de construção em
  produção. Propagação, assinaturas, testes e padrões de `match` não são
  novas emissões. Um validador compartilhado pode atender vários fluxos.
- **Propagar sem embrulhar.** Use `?` ou retorne o `Result` original.
  Não crie wrappers para registrar etapas da pilha. Uma conversão só se
  justifica quando a assinatura exige ou quando acrescenta um fato ou
  decisão observável, preservando a causa original.
- **Eliminar subcódigos.** Quando uma saída JSON expõe a causa, Serde
  apresenta o próprio tipo na fronteira. Não mantenha outra identidade
  textual em métodos, atributos ou tabelas. `Causa` e o contrato antigo
  permanecem apenas nos consumidores ainda não migrados.
- **Decisão separada de diagnóstico.** `Acao` (corrigir argumentos,
  relogar, trocar de instalação, conferir a escrita, reportar) é o
  contrato de decisão; a causa é diagnóstico.
- **Sem "efeito incerto".** Uma escrita é descrita por fatos (`enviada`,
  `aplicada`) e pela causa exata; depois de uma falha, a operação lê de
  novo para reconciliar, sem repetir a escrita.
- **Serde sob demanda.** O domínio propaga `Result`, `Option` e `enum` sem
  exigir derives de serialização. Derive `Serialize` apenas para um tipo
  efetivamente escrito em JSON (CLI, arquivo ou protocolo); derive
  `Deserialize` apenas para um tipo efetivamente lido. Tipos persistidos
  nos dois sentidos podem precisar de ambos. Não serializar todo o
  `Result<T, E>` automaticamente nem criar DTOs só para justificar derives.
- **Apresentação na fronteira.** A CLI serializa apenas o dado público
  necessário com Serde ou apresenta a mensagem do tipo. Classificação e
  testes de identidade usam variantes. Strings fixas de JSON ficam nos
  testes da fronteira, para conferir o formato emitido.

## Propagação e origem única

- **Propagar antes de converter.** Se o erro já é o tipo aceito pelo
  consumidor, devolver `Err` com `?` diretamente. Não criar uma variante
  de passagem só para registrar o módulo ou a função intermediária.
- **Erro específico perto de quem decide.** Uma nova variante é justificada
  por um fato distinto e observável ou por uma decisão diferente; não por
  mera mudança de camada. Rust não tem subclasses de erros: preferir
  composição explícita apenas quando necessária, sem perda da causa.
- **Um lugar para levantar cada falha, quando viável.** Centralizar a
  validação que origina o fato e deixar as chamadas propagarem seu
  `Result`, em vez de construir a mesma variante em vários consumidores.
  Não remover validações nas fronteiras de segurança para obter esse
  objetivo artificialmente.
- **Falha da resposta não é falha do DOM.** URL assinada inválida, cookie
  inválido e MIME não HTML pertencem ao transporte; recusa de HTML não
  observado só aparece depois de uma resposta HTML válida.
- **Decisões são locais ao consumidor.** O tipo da falha não deve
  determinar ações automáticas por padrão; `Acao` pública é uma
  apresentação explícita, não motivo para embrulhar ou reclassificar
  erros dentro do domínio.

## Migração

`scripts/migrar_causas.py` (tree-sitter, fora do gate) converteu 83 tipos
cujos braços eram literais ou delegações, sem mudar nenhum nome público.
A suíte de contrato é a prova. Os oito tipos restantes têm braços não
mecânicos e migram à mão, e o piloto de `acao`/`escrita` é `processo
concluir`. O campo `subcodigo` segue publicado (folha do caminho) até os
contratos passarem a `acao` e `causa`.

Segunda passada: 10 tipos mais (inclusive `ErroArvore`, cujo `match`
estava partido em duas funções, e a cascata `ErroProcessoObter` →
concluir, reabrir, enviar, assinar), e `scripts/migrar_interrogacao.py`
trocou 152 `.map_err(Enum::Caso)?` por `?`. Só troca quando a função
envolvente devolve o próprio `Enum` e o `?` não está numa closure.

Restam à mão:
- **Variantes com `&'static str`** (o leitor devolve o subcódigo como
  texto): `FalhaAnexar::Estrutura`, `FalhaRelacionar::Estrutura` e o
  padrão aninhado de `processo_andamento_registrar::Erro`.
  `ErroCriar::Transporte` já guarda a causa (`FalhaTransporte`,
  `ErroSubmissaoFormulario`); os motivos da releitura da ciência são
  `VerificacaoDaCiencia` e `VerificacaoDaCienciaDoDocumento`.
- **Etapas achatadas** (tipo interno repetido): resolvidas com um tipo
  próprio por etapa, declarado junto de quem levanta o erro e composto por
  `From` (`CookieIlegivel`, `CredenciaisIlegiveis`, `OrigemIlegivel`,
  `SessaoGravadaInvalida`, `HistoricoNaoPedido`, `ErroDasAnotacoes`).
  Variante `transparente` mantém o motivo concreto como nome público.

Terceira passada (10/11/2026): o derive deixou de aceitar
`#[causa(nome = "...")]`. Os 272 atributos saíram; o nome público é o
`snake_case` da variante, e para mudá-lo renomeia-se a variante. O relogin
das leituras passou a decidir por `#[causa(sessao_encerrada)]` em vez de
comparar o texto do subcódigo.

## Forma não observada

Um elemento ausente, repetido, com valor fora da forma ou um HTML
ilegível não sustenta decisão: só diz onde a leitura da tela falhou.
Por isso é um tipo só, `ErroHtml { onde, problema, mensagem }`
(`src/erro_html.rs`), e não uma variante por leitor.

- `onde` é o diagnóstico e a folha pública: o `subcodigo` legado não
  muda. `problema` (`ausente`, `repetido`, `valor`, `html`, ou
  `atributo` com o `ErroAtributoHtml` que subiu) vai no caminho, depois
  de `html`.
- A mensagem é a genérica do problema (`ErroHtml::ausente("onde")`);
  `.com_mensagem("...")` só onde o ponto tem texto próprio. A igualdade
  compara `onde` e `problema`, não a mensagem.
- Um `enum` com outras causas compõe com `#[causa(transparente)]
  Html(ErroHtml)`: o `From` derivado deixa `?` e `.into()` converterem, e
  o caminho não repete `html`. Um leitor que só falha por forma devolve
  `ErroHtml`.
- A variante de forma que algum código casa fora de teste (`match` em
  fluxo, `matches!`, `==`, conversão entre tipos) sustenta uma decisão e
  continua sendo variante.
- **Só nasce de DOM; URL, cabeçalho, arquivo, configuração, entrada e
  domínio têm variante própria.** `ErroHtml` é só o que nasce lendo a
  tela servida (`tl::`, `html_atributos`, leitores de tela, árvore e
  barra). URL e `Location`, cabeçalho HTTP, arquivo local, configuração,
  validação de entrada e fato do domínio ficam no `enum` do módulo onde
  nascem, ainda que o nome diga `Ausente` ou `Invalido`. Redirecionamento
  ou `action` fora da instalação é recusa de segurança, não forma.
- **Leitor novo não declara `enum` de forma: devolve `ErroHtml`.**

`scripts/migrar_erro_html.py` (tree-sitter, com o compilador como
oráculo) migrou 350 variantes de 68 tipos e eliminou 4 tipos inteiros. Os
77 `map_err(|_| ...)` cuja origem era `ErroAtributoHtml` passaram a
carregar a causa. As 21 variantes casadas fora de teste ficaram. A
migração escolheu pelo nome, não pela origem; as variantes de proxy,
`Location`, cabeçalho do anexo, diretório local, id de entrada e fato do
domínio voltaram à forma de antes. O script agora só toma a variante cujo
todo ponto de construção fora de teste está numa `fn` que lê DOM ou num
módulo de leitura de tela (`MODULOS_DE_TELA`).

## Consequências

- Erro novo: tipo perto da detecção, uma origem em regra, propagação
  direta e serialização na fronteira. Não introduzir subcódigos.
- Tabelas `subcodigos-*.md`, o gate `subcodigos` e o trait `Causa`
  com caminho/folha/ação foram removidos (10/11/2026). A saída pública
  é `codigo` + `erro` (mensagem) + HTTP; o derive só gera `From`,
  `status_http()` e `sessao_encerrada()`.
- `?` em vez de `map_err` para compor causas; um `From` manual só onde
  a origem não é um erro do projeto (`io`, `rusqlite`).
