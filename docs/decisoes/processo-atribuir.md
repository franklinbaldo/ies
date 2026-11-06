---
type: ImplementationDecision
title: Leitura estrita da tela de atribuição de processo
operation: processo.atribuir
implementation: partial
status: adopted-for-scope
scope: prévia Web e transporte injetável (POST único e verificação por leitura); --confirmar público permanece bloqueado antes da sessão
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
sem requisições. A prévia foi extraída para um fluxo injetável com testes
 que afirmam 4 GET e 0 POST, e a CLI preserva status HTTP e mensagem saneada.
O fluxo interno de transporte agora envia um POST no máximo uma vez,
reabre a árvore e compara a atribuição na unidade e no processo originais.
Falha do POST não gera retry. Se a releitura não comprovar exatamente o
pedido, a operação devolve efeito incerto (saída 3). A revisão da PR #316
identificou um risco adicional: **releitura coincidente depois de HTTP 403
ou validação da Infra não prova que o nosso POST escreveu**; outro agente
pode ter modificado o processo no intervalo. A resposta recusada não
pode ser reconciliada como sucesso, mesmo com o estado esperado. O mesmo
vale para 302 sem ação e identidade compatíveis. O 302 para
`arvore_visualizar` **também** precisa do `id_procedimento` explícito;
não basta indicar a ação. Essa exigência protege atribuir e remover de
falso sucesso por ação concorrente, conforme RED de duas regressões
adversariais no [CI 42214490209](https://github.com/franklinbaldo/ies/actions/runs/42214490209)
(**2 FAIL novos, 1.716 PASS, 23 ignored existentes**) e observação
do destino `ACEITE` com ID nos testes do Todos @323eb01. Apenas uma resposta
aceita ou uma falha sem resposta de rede pode ser conciliada pela leitura,
conservando a causa; não há nova escrita nem navegação pelo Location.
Os testes injetados afirmam 5 GET e 1 POST (6 tentativas) a frio, tanto
em atribuição quanto em remoção, e as regressões de concorrência exigem
a mesma contagem exata. A CLI pública `--confirmar` continua recusada antes da sessão
porque **replay específico e validação ao vivo não foram executados**.
Não confundir a cobertura sintética com autorização para enviar.

## Evidências verificáveis

- [Contrato](../../contracts/cli/processo-atribuir.md)
- [Evidências da matriz](../../contracts/evidencias/processo-atribuir.md)
- [Teste da trava de escrita](../../tests/cli_processo_atribuir.rs)
- [Testes de forma e payload](../../src/processo_atribuir/tests.rs)
- [Testes de transporte da prévia](../../src/processo_atribuir/tests_fluxo.rs)


[Formulário real no Todos](https://github.com/franklinbaldo/todos/blob/323eb01/tests/paginas_reais/processo_procedimento_atribuicao_cadastrar.html),
[casos de teste](https://github.com/franklinbaldo/todos/blob/323eb01/tests/test_ciclo_atribuir_010.py)
e [cassette UnB](../../cassettes/unb/processo-atribuir-unb--bf0d4327d7d9.md).
Os testes Rust são sintéticos e documentam a origem acima de cada caso
derivado; não comprovam POST nem efeito no SEI.

## Limitações e revisão

`--confirmar` ainda recusa `escrita_de_atribuicao_ainda_nao_verificada`
antes de abrir sessão. O fluxo injetável possui testes de POST único,
releitura e saída 3, mas a decisão será revista após replays e validações
em duas instalações de treinamento. O RED observou 4 testes falhando,
1.689 aprovados no [CI 42195656454](https://github.com/franklinbaldo/ies/actions/runs/42195656454).
