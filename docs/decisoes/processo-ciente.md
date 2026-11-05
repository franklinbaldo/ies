---
type: ImplementationDecision
title: Ciência no processo com GET único e prova na lista
operation: processo.ciente
implementation: partial
status: adopted-for-scope
scope: abertura Web, prévia, GET único e reconciliação por atividade na Lista de Ciências
selected_strategy: arvore-link-servido-funcao-servida-unico-get-lista-relida
alternatives: arvore-link-servido-funcao-servida-unico-get-lista-relida,assumir-sucesso-pelo-html-do-aceite,usar-historico-generico,rest-wssei
evidence_level: documentacao-todos-e-testes-sinteticos-red-green
live_validation: not-run
---

# Ciência pela Lista de Ciências do SEI — #271

## Contexto

O SEI oferece `cienciaProcesso()` na barra da árvore, mas
`procedimento_ciencia` é um **GET mutador**, não uma tela de prévia.
O aceite HTML atualiza a árvore e publica o identificador
`id_atividade` da ciência. Essa resposta não prova o registro:
a lista de ciências relida deve conter o checkbox dessa atividade.
Para documento, há ação diferente (`documento_ciencia`) e marca de
recarga diferente, sem compartilhar uma conclusão indevida.

## Decisão

Conferir primeiro a identidade NUP ↔ id e unidade na abertura Web,
e o botão servido na barra. Sem `--confirmar`, não abrir o GET de
ciência. Com confirmação, abrir a tela do nó, ler a URL da própria
função sem executar JavaScript, e fazer uma única requisição mutadora.
Depois da tentativa, mesmo em timeout ou 302, reabrir a árvore uma vez.
Só publicar sucesso se a nova lista da própria árvore contiver
a atividade exata do aceite. Quando o aceite não identifica atividade
ou a verificação falha, sair com **3**, não repetir o GET mutador.

O leitor das tags utiliza `html_atributos`; o dos literais não usa
regex nem executa JS. Reaproveitar a abertura, o contador de
requisições e o transporte de `processo concluir`. Os endereços
permanecem redigidos, dentro da origem autorizada. Toda resposta
externa exige conferência da identidade, não inferência da URL.

## Alternativas rejeitadas

Aceitar o HTML 200 do GET como sucesso: o próprio SEI pode recusá-lo
com validação. Ler andamento genérico: não prova o registro do usuário
na atividade informada. Tentar novamente após timeout: pode duplicar
a escrita. Montar URL: perde a assinatura da sessão. REST/WSSEI:
sem observação que fundamentasse sua seleção.

## Limitações

O texto da função foi visto na documentação empírica do Todos e a
simulação local não substitui a leitura das cassettes completas. A
segunda ciência do dia deve ser recusada pelo SEI, mas o seu erro
tipado e o motivo ainda exigem captura e testes. A verificação
do usuário da sessão não pode ser afirmada só pelo valor do checkbox;
o fluxo comprova somente a presença da atividade vinculada ao processo.
A CLI de documento ainda não está integrada. Testes de rede sintéticos,
replay e validação live permanecem pendentes.
