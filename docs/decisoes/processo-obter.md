---
type: ImplementationDecision
title: Ler processo pela caixa (resumo), pela árvore com índice local e pelo histórico
operation: processo.obter
implementation: partial
status: adopted-for-scope
scope: resumo na caixa sem abrir o processo; leitura do primeiro nível da árvore abrindo o processo pelo caminho mais barato do índice local (Refs #59, fatia 1); andamentos pelo link do histórico publicado na árvore, visão servida ou completa (fatia 2)
selected_strategy: web-indice-pesquisa-rapida-arvore
alternatives: web-indice-pesquisa-rapida-arvore,web-pesquisa-avancada-arvore,web-caixa-somente,rest-wssei,web-historico-link-da-arvore,web-historico-url-montada
evidence_level: capturas-privadas-estrutura-cassettes-go-e-testes-sinteticos
live_validation: pass-unb-go-pen1-fatia1-not-run-fatia2
---

# Ler processo: resumo na caixa e árvore com índice local

## Alternativas consideradas

- **web-indice-pesquisa-rapida-arvore** (escolhida para a leitura pela
  árvore): link da árvore guardado → entrada por id → pesquisa rápida
  guardada → Controle (linha do NUP exato ou pesquisa rápida do
  formulário), depois tela do processo e árvore.
- **web-pesquisa-avancada-arvore:** o fluxo de `processo buscar`
  (Controle → tela de pesquisa → POST) e o 302 da pesquisa avançada.
- **web-caixa-somente:** abrir só processos com linha no Controle.
- **rest-wssei:** módulo REST opcional, ausente em muitas instalações
  ([ADR 0001](../arquitetura/0001-rotas-por-evidencia.md)).

O `--resumo` continua a decisão anterior: um GET da caixa, sem abrir o
processo nem enviar POST.

## Comparação

- **Pesquisa avançada rejeitada.** Medição do agente pai em 03/11/2026:
  em GO e na PEN1 a pesquisa avançada não achou NUPs que a pesquisa
  rápida, com o NUP exato, abriu (302 para `procedimento_trabalhar`
  assinado, também na UnB). Ela ainda custa uma requisição a mais
  (Controle, tela, POST) e devolve listas que o comando teria de
  interpretar.
- **Só a caixa** não abre processos fora da unidade e faria a ausência
  parecer inexistência.
- **Pesquisa rápida** é um POST de um campo para o `action` assinado do
  `form#frmProtocoloPesquisaRapida`, servido no Controle (UnB, GO, PEN1,
  PEN3, PEN4, UFRJ) e na tela do processo (UnB, GO). O 302 observado nas
  cassettes de GO tem `acao_origem=protocolo_pesquisa_rapida` e o id em
  `id_protocolo`; NUP inexistente respondeu 200 com a tela de pesquisa
  (que o IES não lê).
- **Índice local** ([ADR 0003](../arquitetura/0003-indice-local.md)):
  dentro da mesma sessão o link da árvore é reaberto com um GET; com o id
  conhecido, a entrada sem assinatura medida na UnB custa três GET; a
  recusa dela derruba a sessão de todos, por isso é registrada e o
  comando para pedindo `ies login`.

Orçamento por caminho: link da árvore **1**; entrada por id **3**;
pesquisa rápida guardada **3**; linha do Controle **3**; pesquisa rápida
do Controle **4**. Um palpite que falha soma suas requisições.

## Decisão

`ies processo NUP` sem `--resumo` segue a ordem aprovada pelo
proprietário, definida no
[contrato da árvore](../../contracts/cli/processo-obter-arvore.md), lê
a árvore com um leitor de literais JS escrito à mão (sem regex, sem
executar JavaScript) e publica só o observado: identidade do nó
`PROCESSO`, `documentos` do primeiro nível sem a dica (dado pessoal em
e-mails), `unidade` da tela que a mostrou (`null` no link da árvore) e
`nao_lidos`. O índice aprende NUP ↔ id e os links da tela do processo, da
árvore e da pesquisa rápida; palpite errado é esquecido e o comando segue.

**Efeito declarado.** O GET da árvore marcou como visualizado, em GO, um
processo recebido não visualizado; o de `procedimento_trabalhar` sozinho
não. O IES não restaura esse estado. Lançamento de andamento: não medido.

## Evidências verificáveis

- [Contrato da operação](../../contracts/cli/processo-obter.md)
- [Matriz por instalação](../../contracts/evidencias/processo-obter.md)
- [Árvore do processo](../como-o-sei-funciona/arvore-do-processo.md)
- [Testes de superfície](../../tests/cli_contract.rs)
- Testes do fluxo com transporte roteirizado e índice temporário em
  `src/processo_obter/tests_fluxo.rs` e `tests_indice.rs`; leitor da
  árvore em `src/arvore_processo/tests*.rs`.
- Capturas privadas `0600` (UnB e GO) lidas só pela estrutura: tela do
  processo, árvore e o formulário rápido e as linhas do Controle de seis
  instalações passaram pelos leitores; nenhum valor foi copiado.

## Andamentos (fatia 2)

**Alternativas.** (a) **web-historico-link-da-arvore** (escolhida): o link
assinado `procedimento_consultar_historico` que a árvore publica, nas duas
formas observadas (função servida na UnB, `onclick` com a URL em GO), e o
POST `verHistorico('P')` do formulário servido para a visão completa.
(b) **web-historico-url-montada**: montar a URL do histórico pelo id;
rejeitada, porque sem `infra_hash` a instalação recusa e pode derrubar a
sessão (como na entrada por id), e o IES só monta a entrada por id. (c)
REST: fora da prioridade (ADR 0001). (d) Ler os andamentos da tela do
processo: a tela não os publica.

**Evidência.** Capturas privadas `0600` (UnB 4.0.9 e GO 5.0.3, 03/11/2026)
lidas só pela estrutura passaram pelos leitores: as duas formas do link,
as páginas `R` (UnB) e `P` (GO, 49 e 17 linhas), a regra servida de
`verHistorico` e a derivação do POST pelo primitivo de formulário (dez
ocultos, nenhum botão). Medições do agente pai: a visão `P` de 14
processos de GO veio inteira até 75 linhas, sem paginador; a ablação na
UnB mostrou que `hdnInfraNroItens` e `hdnInfraPaginaAtual` não paginam o
histórico. Duas árvores de GO não publicaram o link: isso sai como
`andamentos: null` com motivo, não como falha. O leitor de linhas de
`--historico-privado` (`src/historico.rs`) leu as páginas reais sem
mudança.

**Orçamento.** Padrão +1 GET; `--andamentos-completos` +1 GET +1 POST
(o POST só se o GET serviu `R`); `--sem-andamentos` +0. Repetição quente
no caminho 1: árvore e histórico guardado, **2**. Tabela por caminho no
[contrato dos andamentos](../../contracts/cli/processo-obter-andamentos.md).

**Efeitos.** O GET do histórico é leitura (lançamento de andamento não
medido). O POST `P` é declarado `PaginaOuFiltroLembrado`: não se mediu
se a visão fica lembrada. A visão não é restaurada (vários agentes na
mesma conta se desfariam). O botão `btnAtualizarAndamento` (escrita) nunca
participa. Falha HTTP no histórico (503 visto em GO) encerra o comando com
o erro de transporte, sem retry; o 403 do WAF de GO é
`bloqueio_de_seguranca_da_instalacao`.

**Trade-offs.** Ler sempre a visão servida custa um GET a mais por
comando, mas publica o que a pessoa vê no SEI; o completo exige uma
mudança de estado de sessão declarada. Legenda com faixa de paginação é
recusa até ser observada.

## Limitações e revisão

Validação live: **PASS** na UnB, em GO e na PEN1 nos caminhos 1, 2, 3 e 4a, e na UnB também no 4b ([relatório](../validacao-live/2026-11-03-processo-obter-arvore.md)); recusa da entrada por id NOT_RUN ao vivo. Pastas
(`PASTA`), marcadores e propriedades ficam para as próximas fatias.
Andamentos: validação live **NOT_RUN** pelo IES até o agente pai rodar
UnB, GO e PEN1. Revisar se a legenda trouxer paginação, se outra
instalação publicar o link noutra forma ou se a visão `P` ficar lembrada. Revisar se a entrada por id responder outra forma, se o 302 da
pesquisa rápida mudar de chave do id fora de GO, ou se a árvore publicar
outro construtor.
