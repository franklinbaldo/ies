---
type: CommandContract
title: Prévia experimental de alteração da especificação de processo
description: Consulta a especificação existente e prepara a comparação sem enviar a alteração cadastral
operation: processo.alterar
command: ies processo alterar NUP --especificacao TEXTO [--confirmar]
effect: write-process-state
implementation: partial
---

# Processo alterar (#877)

A primeira fatia é exclusivamente a prévia experimental. A CLI exige o
link de alteração publicado na árvore, identifica o NUP e o ID na tela
de propriedades, e confere a action do formulário na mesma instalação e
processo. Tela apenas de consulta, cadastro individual ou sigiloso são
recusados nesta fatia.

A comparação usa a especificação lida e o texto solicitado. A prévia
não envia o formulário cadastral e não atesta que o SEI aceitaria o
texto, nem que os demais campos seriam preservados por uma futura escrita.
Se os textos forem iguais, informa que nenhuma alteração é necessária.
A abertura de processo segue os efeitos já documentados da leitura Web.

A flag `--confirmar` sempre recusa com saída 1 e
`EscritaAindaNaoValidada`, inclusive no caso de texto igual, antes de
abrir a sessão. Não há POST de alteração, retry ou promessa de escrita.

## Saída JSON da prévia

```json
{"numero":"00000.000001/2026-01","id_procedimento":"91",
 "especificacao_anterior":"Anterior","especificacao_pedida":"Nova",
 "alteracao_necessaria":true,"alterado":false,"confirmado":false,
 "origem":"web","requisicoes":4}
```

`requisicoes` é a contagem observada, não um número fixo. A saída
textual distingue alteração necessária e no-op. Nenhuma URL assinada
ou credencial é publicada.

## Evidência e limite

Os [cards da operação](../evidencias/processo-alterar.md) são evidência
de demonstração do Todos, não de escrita do IES. A preparação do corpo
é hipótese; não foi comprovada a semântica dos rótulos do JavaScript
original (NBSP e espaços). A confirmação fica para outra fatia, com
replay original aplicável, confronto independente de IDs e multiplicidade,
POST único e releitura do cadastro. Live NOT_RUN.
