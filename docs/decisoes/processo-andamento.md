---
type: ImplementationDecision
title: Andamento do processo com POST único e prova por histórico
operation: processo.andamento.registrar
implementation: partial
status: experimental
scope: ies processo andamento NUP --texto T
selected_strategy: web-formulario-servido-com-releitura
alternatives: web-formulario-servido-com-releitura,web-sem-conferencia,rest-wssei
evidence_level: todos-unb-e-cassettes-de-treinamento
live_validation: not-run
---

# Decisão de registro de andamento (#259)

## Observação

O Todos, em src/todos/andamento.py no commit 323eb01, documenta na UnB o link procedimento_atualizar_andamento, formulário frmAtividadeListar, descrição txaDescricao, dois controles de identidade e botão sbmSalvar. Aceite observado: 302 para procedimento_consultar_historico. Limite de 500 caracteres é imposto pelo SEI.

## Escolha

Conferir o id na árvore e no formulário antes do POST. Usar o primitivo Web com efeito declarado, um único envio e nova leitura. Confirmar somente acréscimo de uma linha no topo, com todas as observadas deslocadas e total acrescido de um. Texto igual não é identificador. 302 não é prova; releitura obrigatória mesmo se houver falha da resposta.

Descartados: restaurar a página, seguir 302, retry, REST ou SOAP, e considerar HTTP 200/302 suficiente.

## Revisão e limites

A lista resumida pode truncar o fim. Por isso se compara o prefixo observado. Mudanças concorrentes na mesma conta podem tornar a prova impossível; efeito incerto tem saída 3. Falta teste integrado do transporte, replay e validação real na UnB. Nenhum POST remoto executado nesta sessão.
