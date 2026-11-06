---
type: ImplementationDecision
title: Leitura estrita da tela de atribuição de processo
operation: processo.atribuir
implementation: partial
status: adopted-for-scope
scope: prévia Web de ies processo atribuir (seleção estrita e corpo POST derivado sem envio); --confirmar recusa antes da sessão nesta fatia
selected_strategy: usuario-exato-da-lista-e-arvore-como-fonte
alternatives: usuario-exato-da-lista-e-arvore-como-fonte,substring,html-sem-conferencia,rest-wssei
evidence_level: pagina-real-todos-323eb01-testes-sinteticos
live_validation: not-run
---

# Atribuir processo — fatia de leitura (Refs #264)

A página de atribuição observada na UnB usa `frmAtividadeAtribuir` com
`selAtribuicao`, opção `null` para remover, `hdnIdProtocolo` e
`sbmSalvar`. O `select` não marca quem está atribuído: esse dado está na
situação `aberto_em` do nó do processo, específica da unidade.

## Alternativas consideradas

- `usuario-exato-da-lista-e-arvore-como-fonte` (escolhida): login inteiro
  na lista `selAtribuicao`; o atribuído atual vem de `aberto_em` na árvore.
- `substring`: casar parte do nome ou do login. Descartada: ambígua.
- `html-sem-conferencia`: enviar o formulário sem conferir action,
  identidade e botão. Descartada: sem prova de identidade.
- `rest-wssei`: fora do escopo Web do IES.

## Comparação

Só a escolhida evita ambiguidade e não depende de um estado que o
`select` não publica.

## Decisão

A primeira fatia implementa um leitor HTML estrito, reutilizando
`html_atributos` e `localizar_formulario`, sem executar JavaScript.
Conferir um único formulário POST, `action` assinado do processo aberto,
único botão Salvar, campo oculto de identidade, lista simples, opção
`null` habilitada e não ambígua. Escolher usuário por **login inteiro**,
sem buscar por substring ou nome, e recusar estado não observado ou
atribuição já aplicada. A função de planejamento é pura e não envia POST.

A prévia Web já usa esse leitor. Nesta fatia, ela também deriva a submissão
com o primitivo único `formulario_web`, exigindo que os ocultos servidos,
o botão Salvar e a opção `selAtribuicao` produzam exatamente o corpo do
navegador. Rejeita um mesmo ID de usuário associado a dois logins. Os
testes sintéticos conferem o payload de atribuição e o `null` de remoção,
sem requisições. O POST real, a releitura do efeito e o ensaio ao vivo
**ainda não foram implementados**. `--confirmar` permanece fechado antes
da sessão. Não confundir a preparação verde com autorização para enviar.

## Evidências verificáveis

- [Contrato](../../contracts/cli/processo-atribuir.md)
- [Evidências da matriz](../../contracts/evidencias/processo-atribuir.md)
- [Teste da trava de escrita](../../tests/cli_processo_atribuir.rs)
- [Testes de forma e payload](../../src/processo_atribuir/tests.rs)


[Formulário real no Todos](https://github.com/franklinbaldo/todos/blob/323eb01/tests/paginas_reais/processo_procedimento_atribuicao_cadastrar.html),
[casos de teste](https://github.com/franklinbaldo/todos/blob/323eb01/tests/test_ciclo_atribuir_010.py)
e [cassette UnB](../../cassettes/unb/processo-atribuir-unb--bf0d4327d7d9.md).
Os testes Rust são sintéticos e documentam a origem acima de cada caso
derivado; não comprovam POST nem efeito no SEI.

## Limitações e revisão

`--confirmar` recusa `escrita_de_atribuicao_ainda_nao_verificada` antes
de abrir a sessão. A decisão será revista quando houver testes de
transporte do POST único, releitura e saída 3, além de replay e live em
treinamento.
