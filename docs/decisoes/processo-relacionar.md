---
type: ImplementationDecision
title: Prévia e remoção única derivadas da tela do relacionamento
operation: processo.relacionar
implementation: partial
status: adopted-for-scope
scope: ies processo relacionar NUP --com NUP com prévia e remoção
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

A remoção confirmada deriva a action do handler `acaoRemover` e exige POST
único com 302 válido e leitura independente sem a linha. A adição envia o
botão `sbmAdicionar` com os três campos publicados, preenchidos pelo
complementar XML, e só confirma diante de 302 de aceite e releitura
independente contendo id e NUP do destino. Qualquer falha no POST
ou evidência inconclusiva dá saída 1 sem retry. Esta evidência PEN1
não presume paridade nas demais instalações. Live NOT_RUN.

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

A fatia implementa prévia estrutural, consulta complementar única, adição
e remoção com POST único e reconciliação independente, validadas por
testes sintéticos. O replay completo e a validação live em duas
instalações permanecem pendentes.

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
A remoção e a adição confirmadas usam POST único derivado e verificação
independente em ensaios sintéticos; resta comprovar em treinamento.
Replay integral e live NOT_RUN; nenhuma requisição real ou resíduo.
