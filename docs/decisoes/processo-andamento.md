---
type: ImplementationDecision
title: Andamento do processo com POST único e prova por histórico
operation: processo.andamento
implementation: partial
status: adopted-for-scope
scope: prévia Web e confirmação exclusivamente em instalações autorizadas de treinamento ou simulador local, com um POST e releitura
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
- Testes sintéticos em `src/processo_andamento_registrar/tests_fluxo.rs` e E2E CLI real → HTTPS loopback → anti-IES → SQLite em `tests/aies_andamento_cli.rs` para BDD-259-01/02.

## Limitações e revisão

Desde 10/11/2026, aprovado no ensaio da UnB de treinamento, `--confirmar` grava também em produção (decisão do proprietário).
A prova BDD-259-02 é sintética: duas CLIs consultam o mesmo SQLite após um POST único; integridade UnB/GO verificada, mas confronto semântico independente e live permanecem NOT_RUN. A lista resumida pode truncar o fim. Por isso se compara o prefixo observado. Mudanças concorrentes na mesma conta podem tornar a prova impossível; efeito incerto tem saída 3. Falta teste integrado do transporte, replay e validação real na UnB. Nenhum POST remoto executado nesta sessão.
