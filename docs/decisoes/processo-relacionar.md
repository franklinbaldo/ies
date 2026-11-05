---
type: ImplementationDecision
title: Prévia do relacionamento pelo formulário e complementar servidos
operation: processo.relacionar
implementation: partial
status: adopted-for-scope
scope: ies processo relacionar NUP --com NUP com prévia apenas
selected_strategy: web-barra-formulario-complementar-e-linhas
alternatives: web-barra-formulario-complementar-e-linhas,rest,html-solto
evidence_level: tela-todos-pen1-e-testes-sinteticos
live_validation: not-run
---

# Relacionar processos — recorte #258

O Todos @323eb01 é fonte dos casos, não da arquitetura. A tela de
relacionamento PEN1 demonstra o complementar protocolo_RI1023, a tabela
Lista de Processos Relacionados e a remoção vinculada à linha. A cassette
PEN1 registra a operação sem que seu corpo HTTP tenha sido reproduzido
integralmente nesta execução.

Primeira fatia: ler a barra e a tabela via html_atributos, identificar destino
pela consulta AJAX de leitura com formulário_web, rejeitar autocorrelação e
repetições. Só uma linha da tabela é evidência de vínculo: não buscar
números livres pelo HTML.

Confirmação continua bloqueada antes da sessão: falta implementar e testar
POST único e releitura independente para adição e remoção, com saída 3 se
efeito incerto. Nenhum retry nem desligamento de gate é permitido. Esta
evidência PEN1 não presume paridade nas outras instalações. Live NOT_RUN.

## Alternativas consideradas

1. A Web publicada pela barra, formulário e consulta complementar:
   caminho escolhido, conservando a fonte do link e identidade.
2. Varredura livre de NUPs na página: rejeitada por falsos positivos.
3. REST/SOAP: rejeitado por não ser a interface escolhida pelo IES.

## Comparação

A Web permite provar o NUP da linha do próprio relacionamento e os campos
que serão necessários ao POST. Procurar números em texto solto não prova
vínculo; REST e SOAP não compartilham as invariantes de autenticação Web
da CLI.

## Decisão

A fatia inicial implementa só prévia, com parser estrutural e consulta
complementar única. Escrita, replay e validação de outras instalações
ficam pendentes; --confirmar falha fechado antes de abrir sessão.

## Evidências verificáveis

- [Contrato de CLI](../../contracts/cli/processo-relacionar.md).
- [Evidência da operação](../../contracts/evidencias/processo-relacionar.md).
- [Teste CLI](../../tests/cli_processo_relacionar.rs).
- [Casos de tela e XML](../../src/processo_relacionar/tests.rs).
- [Relatório de execução](../validacao-live/2026-11-05-processo-relacionar-handoff.md).

## Limitações e revisão

O parser PEN1 não prova paridade nas demais instalações, nem o número
exato de pedidos na primeira abertura de um processo sem índice. O GET
da tela e a consulta POST (leitura) são distintos da futura escrita.
Antes de liberar --confirmar: observar a resposta do SEI, implementar
POST único derivado e testar verificação independente com saída 3
para efeito incerto. Replay e live NOT_RUN; nenhum resíduo.
