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
garante que o `match` cobre os erros, mas não que o texto corresponda
ao erro. Por isso existiam tabelas de subcódigos longe do código e um
hook só para mantê-las em sincronia. Nenhum consumidor programático decide
pelo `subcodigo`: os únicos usos eram testes e o próprio verificador.

## Decisão

- **O tipo é a verdade.** Declare o erro perto do ponto que detecta a
  falha. Em regra, cada erro tem um único ponto de construção em
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
- **Sem "escrita não confirmada".** Uma escrita é descrita por fatos (`enviada`,
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
  testes de identidade comparam o erro. Strings fixas de JSON ficam nos
  testes da fronteira, para conferir o formato emitido.

## Propagação e origem única

- **Propagar antes de converter.** Se o erro já é o tipo aceito pelo
  consumidor, devolver `Err` com `?` diretamente. Não criar um erro
  de passagem só para registrar o módulo ou a função intermediária.
- **Erro específico perto de quem decide.** Um novo erro é justificado
  por um fato distinto e observável ou por uma decisão diferente; não por
  mera mudança de camada. Rust não tem subclasses de erros: preferir
  composição explícita apenas quando necessária, sem perda da causa.
- **Um lugar para levantar cada falha, quando viável.** Centralizar a
  validação que origina o fato e deixar as chamadas propagarem seu
  `Result`, em vez de construir o mesmo erro em vários consumidores.
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
- **Erros com `&'static str`** (o leitor devolve o subcódigo como
  texto): `FalhaAnexar::Estrutura`, `FalhaRelacionar::Estrutura` e o
  padrão aninhado de `processo_andamento_registrar::Erro`.
  `ErroCriar::Transporte` já guarda a causa (`FalhaTransporte`,
  `ErroSubmissaoFormulario`); os motivos da releitura da ciência são
  `VerificacaoDaCiencia` e `VerificacaoDaCienciaDoDocumento`.
- **Etapas achatadas** (tipo interno repetido): resolvidas com um tipo
  próprio por etapa, declarado junto de quem levanta o erro e composto por
  `From` (`HistoricoNaoPedido`, `ErroDasAnotacoes`).
  O erro `transparente` mantém o motivo concreto como nome público.
- **Arquivo privado fora do transporte** (12/11/2026): `CookieIlegivel`,
  `CredenciaisIlegiveis`, `OrigemIlegivel` e `SessaoGravadaInvalida`
  tinham uma variante só, que embrulhava a `FalhaTransporte` de uma
  leitura de arquivo. A leitura (`arquivo_privado::ler`) devolve
  `ErroArquivoPrivado`, um fato por variante, e quem lê embrulha uma vez
  com o nome do arquivo: `FalhaTransporte::CookieDaSessaoIlegivel`,
  `ErroLogin::ArquivoDeCredenciais`. Os comandos recebem o cookie que a
  abertura leu e conferiu (`CookieLido`, em `AcoesDeEstado`,
  `DocumentoNaArvoreAberto`, `DocumentoObtido` e `Troca`): a variante
  `Cookie` dos comandos, que nenhum caminho produzia, saiu. Só `processo
  criar` lê o cookie primeiro, e cada fato do retrato é variante de
  `ErroCriar` (`CookieDaSessaoIlegivel`, `CookieForaDoFormato`,
  `ControleForaDaOrigem`).

Terceira passada (10/11/2026): o derive deixou de aceitar
`#[causa(nome = "...")]`. Os 272 atributos saíram; o nome público é o
`snake_case` do erro, e para mudá-lo renomeia-se o erro. O relogin
das leituras passou a decidir por `#[causa(sessao_encerrada)]` em vez de
comparar o texto do subcódigo.

## Forma não observada

Um elemento ausente, repetido, com valor fora da forma ou um HTML
ilegível é um erro do leitor que o detecta, declarado ao lado dele e
nomeado pelo fato: `FormularioAusenteNaTela`,
`BarrasDoProcessoRepetidasNaArvore`, `LinhaDoControleIlegivel`. Quem
consome o leitor embrulha o erro dele com `#[from]` e não o
re-traduz.

- O erro nasce onde é detectado. A barra do nó `PROCESSO` é lida num
  lugar (`ancoras_da_barra`) e falha com `ErroBarraDoProcesso`; as
  ações que leem links dessa barra (`ErroConcluirNaBarra`,
  `ErroEnviarNaBarra`, …) carregam `Barra(#[from] ErroBarraDoProcesso)`
  em vez de cada uma repetir as duas variantes.
- Não há tipo genérico de forma nem campo que diga o lugar por texto.
  `ErroHtml { onde, problema, mensagem }` existiu até 10/11/2026 e foi
  removido por ablação: apagado o tipo, o compilador apontou cada uso,
  e cada um virou a variante do leitor que detecta a falha.
- A causa do tokenizador (`ErroAtributoHtml`) sobe como `#[source]`
  onde o leitor a recebe.
- Os mesmos limites de antes valem: só é forma o que nasce lendo a tela
  servida. URL e `Location`, cabeçalho HTTP, arquivo local, configuração,
  validação de entrada e fato do domínio ficam no `enum` do módulo onde
  nascem. Redirecionamento ou `action` fora da instalação é recusa de
  segurança, não forma.

## Consequências

- Erro novo: tipo perto da detecção, uma origem em regra, propagação
  direta e serialização na fronteira. Não introduzir subcódigos.
- Tabelas `subcodigos-*.md`, o gate `subcodigos` e o trait `Causa`
  com caminho/folha/ação foram removidos (10/11/2026). A saída pública
  é `codigo` + `erro` (mensagem) + HTTP.
- O derive próprio (`ies-derive`) saiu (#674): os erros usam `thiserror`.
  `#[from]` gera o `From`; `#[source]` liga a causa. `status_http()` e
  `sessao_encerrada()` percorrem a cadeia `source()`, achando
  `StatusHttpRecusado` ou `erro_publico::SessaoEncerrada` por `downcast`.
  Não usar `#[error(transparent)]`: ele esconde o nó interno da cadeia.
- `?` em vez de `map_err` para compor causas; um `From` manual só onde
  a origem não é um erro do projeto (`io`, `rusqlite`).
