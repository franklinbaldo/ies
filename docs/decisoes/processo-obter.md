---
type: ImplementationDecision
title: Ler processo pela árvore com índice local e pelo histórico
operation: processo.obter
implementation: partial
status: adopted-for-scope
scope: leitura do primeiro nível da árvore abrindo o processo pelo caminho mais barato do índice local (Refs #59, fatia 1); andamentos pelo link do histórico publicado na árvore, visão servida ou completa (fatia 2); pastas abertas por Abrir todas as Pastas e conferidas pela identidade dos protocolos (fatia 3); colheita do Controle no índice e passo 1b pelo link procedimento_trabalhar colhido (fatia 4); processos anexados lidos do nó PROCESSO_ANEXADO da mesma árvore, sem os documentos deles, e o pai do processo anexado lido do link do nó raiz; documentos movidos fora dos documentos, com o destino pelo link do nó; propriedades pelo formulário procedimento_alterar (ou procedimento_consultar) publicado na barra do nó do processo, sem envio; marcadores pela tela andamento_marcador_gerenciar da barra, sempre lista (apêndice)
selected_strategy: web-indice-pesquisa-rapida-arvore
alternatives: web-indice-pesquisa-rapida-arvore,web-pesquisa-avancada-arvore,web-caixa-somente,rest-wssei,web-historico-link-da-arvore,web-historico-url-montada,web-abrir-todas-as-pastas,web-paginar-pasta-por-post,web-colheita-do-controle,web-get-extra-para-o-indice,web-anexados-do-no-da-arvore,web-abrir-cada-anexado,web-vinculo-recusa-fechada,web-vinculo-nulo-com-motivo,web-propriedades-da-barra,web-propriedades-todas-as-telas,web-propriedades-listas-ocultas,web-movidos-pelo-link-do-no,web-movidos-pela-dica,web-marcadores-da-barra,web-marcadores-da-celula-do-controle
evidence_level: capturas-privadas-estrutura-replay-do-acervo-ensaios-live-e-testes-sinteticos
live_validation: pass-unb-go-pen1-todas-as-fatias-ufrj-pen3-pen4-not-run
---

# Ler processo pela árvore com índice local
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

`--resumo` (**web-caixa-somente** num GET) e as opções de captura
privada foram aposentados em 04/11/2026 por decisão do proprietário
([apêndice](processo-obter/opcoes-aposentadas.md)).

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

`ies processo NUP` segue a ordem aprovada pelo
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
  `src/processo_obter/tests_fluxo.rs`, `tests_indice.rs`,
  `tests_pastas.rs` e `tests_colheita.rs` (passo 1b); colheita em
  `src/colheita_do_controle/tests.rs`; leitor da árvore em
  `src/arvore_processo/tests*.rs`.
- [Contrato das pastas](../../contracts/cli/processo-obter-pastas.md)
- [Contrato das propriedades](../../contracts/cli/processo-obter-propriedades.md);
  testes em `src/processo_obter/tests_propriedades.rs`,
  `src/propriedades_processo/tests_tela.rs` e
  `src/arvore_processo/tests_barra.rs`
- Capturas privadas `0600` (UnB e GO) lidas só pela estrutura: tela do
  processo, árvore e o formulário rápido e as linhas do Controle de seis
  instalações passaram pelos leitores; nenhum valor foi copiado.

## Limitações e revisão

Propriedades: validação live **PASS** na UnB e na PEN1 ([PEN1](../validacao-live/2026-11-04-processo-obter-movidos.md), [relatório](../validacao-live/2026-11-04-processo-obter-propriedades.md)).
Validação live: **PASS** na UnB, em GO e na PEN1 nos caminhos 1, 2, 3 e 4a, e na UnB também no 4b ([relatório](../validacao-live/2026-11-03-processo-obter-arvore.md)); recusa da entrada por id NOT_RUN ao vivo. Marcadores:
[apêndice](processo-obter/marcadores.md), validação live PASS na UnB; pastas: validação live
**PASS** na UnB, frio, quente e sessão nova, com o servidor sem lembrar as pastas abertas ([relatório](../validacao-live/2026-11-03-processo-obter-pastas.md)).
Andamentos: validação live **PASS** na UnB e na PEN1 (frio, quente,
completo, `--sem-andamentos`) e em GO (frio), com a visão `P` **lembrada**
pelo servidor ([relatório](../validacao-live/2026-11-03-processo-obter-andamentos.md)).
Colheita e passo 1b: validação live **PASS** na UnB e na PEN1 ([relatório](../validacao-live/2026-11-03-colheita-do-controle.md)).
Revisar se a legenda trouxer paginação ou se outra instalação publicar o
link noutra forma. Revisar se a entrada por id responder outra forma, se o 302 da
pesquisa rápida mudar de chave do id fora de GO, ou se a árvore publicar
outro construtor.

## Partes da decisão

- [Decisão de processo obter — andamentos, pastas e colheita](processo-obter/fatias-de-leitura.md): fatias 2 a 4 da decisão de processo obter, com andamentos, pastas e colheita do Controle.
- [Decisão de processo obter — anexados, propriedades e movidos](processo-obter/arvore-e-metadados.md): processos anexados, propriedades, documentos movidos, situação e natureza na decisão de processo obter.
