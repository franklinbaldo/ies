---
type: ImplementationDecision
title: Incluir documento interno pelo formulário publicado e conferir diferença da árvore
operation: documento.criar
implementation: partial
status: adopted-for-scope
scope: contrato, reconciliação em produção e fluxo WIP de submissão Web com uma escrita
selected_strategy: web-cadastro-e-diferenca-da-arvore
alternatives: web-cadastro-e-diferenca-da-arvore,web-confiar-em-id-da-resposta,rest-wssei
evidence_level: oraculo-todos-captura-unb-e-indices-de-cassette-go
live_validation: not-run
---

# Decisão para `documento criar` (issue #246)

## Contexto

O formulário deve ser observado por instalação, sem supor permissões ou campos.

## Alternativas consideradas

O objetivo é criar um documento nativo vazio, não uma cópia nem um
anexo externo. O Todos usa o link da barra, a lista de tipos e o
cadastro; a captura UnB 4.0.9 mostra `txtDescricao`,
`rdoNivelAcesso` e `lblSerieTitulo`.

1. **Web com formulário servido e diferença de árvores** (escolhida):
   exige um POST, seguido de leitura da árvore e confirmação de um
   único id e número SEI novo, preservando os existentes.
2. **Confiar no `id_documento` do Location**: economiza a conferência,
   mas não prova a presença de um novo documento nem expõe o número SEI.
3. **REST/WSSEI**: ainda sem evidência de disponibilidade e em
   desacordo com a prioridade Web do projeto.

## Comparação

| Critério | Web + árvore | Só Location | REST |
| --- | --- | --- | --- |
| Comprova número SEI na árvore | sim | não | não observado |
| Exige POST de criação | sim | sim | não avaliado |
| Usa URL assinada servida | sim | sim | não avaliado |
| Permite recusar efeito incerto | sim | não | não observado |

## Decisão e limites

Conferir a unidade e a situação antes de escrever, sem deduzir
permissões pelo botão. O tipo é escolhido pelo nome oferecido, com
recusas distintas por ausência e ambiguidade. O formulário real deve
ser derivado pelo primitivo compartilhado com efeito explícito, nunca
por `POST` montado à mão. Não repetir escrita por falha de rede.

A reconciliação não é um teste de quantidade apenas: os ids anteriores
precisam persistir; não podem surgir dois ids novos; o novo deve
publicar número SEI e tipo solicitado. Não aceitar "algum documento
novo" como êxito. Qualquer inconclusividade depois de enviar é 3.

## Evidências verificáveis

- [Contrato](../../contracts/cli/documento-criar.md).
- [Evidência existente do acervo](../../contracts/evidencias/documento-criar.md).
- [Teste CLI](../../tests/cli_documento_criar.rs) (confere argumentos e falha fechada sem sessão).
- Testes unitários de leitura e reconciliação: `src/documento_criar/tests.rs`.
- Testes de derivação de formulário: `src/documento_criar/fluxo.rs`.

## Limitações e revisão

[Contrato](../../contracts/cli/documento-criar.md);
[observação](../como-o-sei-funciona/inclusao-documento-interno.md).
O acervo comprova a existência de interações, mas o conteúdo bruto
ainda não foi reexecutado no IES. Sem replay e validação do cadastro
de cada versão, **não** se deve habilitar escrita por adivinhação.
A decisão deverá ser revista após o RED e a conferência em instalações
de treinamento. A escrita foi autorizada em 05/11, exclusivamente no SEI de treinamento; porém segue **NOT_RUN** nesta execução. O fluxo de envio e conferência é WIP sem compilação, replay ou live comprovados; não declarar GREEN até os gates passarem.
