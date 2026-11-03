---
type: ImplementationDecision
title: Ler processo pela caixa (resumo) e pela árvore com índice local
operation: processo.obter
implementation: partial
status: adopted-for-scope
scope: resumo na caixa sem abrir o processo; leitura do primeiro nível da árvore abrindo o processo pelo caminho mais barato do índice local (Refs #59, fatia 1)
selected_strategy: web-indice-pesquisa-rapida-arvore
alternatives: web-indice-pesquisa-rapida-arvore,web-pesquisa-avancada-arvore,web-caixa-somente,rest-wssei
evidence_level: capturas-privadas-estrutura-cassettes-go-e-testes-sinteticos
live_validation: pass-unb-go-pen1
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

## Limitações e revisão

Validação live: **PASS** na UnB, em GO e na PEN1 nos caminhos 1, 2, 3 e 4a, e na UnB também no 4b ([relatório](../validacao-live/2026-11-03-processo-obter-arvore.md)); recusa da entrada por id NOT_RUN ao vivo. Pastas
(`PASTA`), andamentos, marcadores e propriedades ficam para as próximas
fatias. Revisar se a entrada por id responder outra forma, se o 302 da
pesquisa rápida mudar de chave do id fora de GO, ou se a árvore publicar
outro construtor.
