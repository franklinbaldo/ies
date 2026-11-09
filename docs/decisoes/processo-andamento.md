---
type: ImplementationDecision
title: Andamento do processo com POST único e prova por histórico
operation: processo.andamento
implementation: partial
status: adopted-for-scope
scope: prévia Web para todos; confirmação em treinamento UnB/GO ou simulador .test, inclusive release, com releitura; demais origens recusadas
selected_strategy: web-formulario-servido-com-releitura
alternatives: web-formulario-servido-com-releitura,web-sem-conferencia,rest-wssei
evidence_level: todos-unb-e-cassettes-de-treinamento
live_validation: not-run
---

# Decisão de registro de andamento (#259)

## Alternativas consideradas

- `web-formulario-servido-com-releitura` (escolhida): POST único do formulário servido e prova por nova leitura do histórico.
- `web-sem-conferencia`: aceitar o 302 como prova. Descartada: o 302 não prova a escrita.
- `rest-wssei`: fora do escopo Web do IES.

## Comparação


O Todos, em src/todos/andamento.py no commit 323eb01, documenta na UnB o link procedimento_atualizar_andamento, formulário frmAtividadeListar, descrição txaDescricao, dois controles de identidade e botão sbmSalvar. Aceite observado: 302 para procedimento_consultar_historico. Limite de 500 caracteres é imposto pelo SEI.

## Decisão

Conferir o id na árvore e no formulário antes do POST. Usar o primitivo Web com efeito declarado, um único envio e nova leitura. Confirmar somente acréscimo de uma linha no topo, com todas as observadas deslocadas e total acrescido de um. Texto igual não é identificador. 302 não é prova; releitura obrigatória mesmo se houver falha da resposta.

Descartados: restaurar a página, seguir 302, retry, REST ou SOAP, e considerar HTTP 200/302 suficiente.

## Evidências verificáveis

- [Contrato](../../contracts/cli/processo-andamento.md)
- [Evidências da matriz](../../contracts/evidencias/processo-andamento.md)
- [Teste da trava de escrita](../../tests/cli_processo_andamento.rs)
- Testes sintéticos em `src/processo_andamento_registrar/tests_fluxo.rs`. A confirmação possui teste CLI real no anti-IES e testes unitários do fluxo.

## Limitações e revisão

A confirmação é habilitada somente para as origens exatas UnB/GO de treinamento e o simulador `.test`. A origem é verificada antes da requisição, mesmo no binário de distribuição. O teste BDD-259-02 exercita duas execuções reais da CLI com um SQLite compartilhado, incluindo consulta do histórico completo; comprova somente compatibilidade sintética, não paridade com cassette ou treinamento, ambos NOT_RUN. 
A lista resumida pode truncar o fim. Por isso se compara o prefixo observado. Mudanças concorrentes na mesma conta podem tornar a prova impossível; efeito incerto tem saída 3. Falta teste integrado do transporte, replay e validação real na UnB. Nenhum POST remoto executado nesta sessão.
