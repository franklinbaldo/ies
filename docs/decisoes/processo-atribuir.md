---
type: ImplementationDecision
title: Leitura estrita da tela de atribuição de processo
operation: processo.atribuir
implementation: partial
status: proposed
scope: seletor e plano puros sem rede, preparando fluxo Web de atribuição
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

## Decisão

A primeira fatia implementa um leitor HTML estrito, reutilizando
`html_atributos` e `localizar_formulario`, sem executar JavaScript.
Conferir um único formulário POST, `action` assinado do processo aberto,
único botão Salvar, campo oculto de identidade, lista simples, opção
`null` habilitada e não ambígua. Escolher usuário por **login inteiro**,
sem buscar por substring ou nome, e recusar estado não observado ou
atribuição já aplicada. A função de planejamento é pura e não envia POST.

O fluxo de rede da CLI, a verificação por releitura e o ensaio em
instalações de treinamento **ainda não estão implementados nesta fatia**.
Nenhuma operação remota ocorreu. Não confundir o leitor verde com
autorização para execução sem verificação.

## Evidência e limites

[Formulário real no Todos](https://github.com/franklinbaldo/todos/blob/323eb01/tests/paginas_reais/processo_procedimento_atribuicao_cadastrar.html),
[casos de teste](https://github.com/franklinbaldo/todos/blob/323eb01/tests/test_ciclo_atribuir_010.py)
e [cassette UnB](../../cassettes/unb/processo-atribuir-unb--bf0d4327d7d9.md).
Os testes Rust são sintéticos e documentam a origem acima de cada caso
derivado; não comprovam POST nem efeito no SEI.
