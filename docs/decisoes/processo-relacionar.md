---
type: ImplementationDecision
title: Prévia e confirmação de relacionamento com releitura verificadora
operation: processo.relacionar
implementation: partial
status: adopted-for-scope
scope: ies processo relacionar NUP --com NUP com escrita confirmada
selected_strategy: web-barra-formulario-complementar-e-linhas
alternatives: web-barra-formulario-complementar-e-linhas,rest,html-solto
evidence_level: tela-todos-pen1-e-testes-sinteticos
live_validation: not-run
---

# Relacionar processos — recorte #258

O Todos @323eb01 é fonte dos casos, não da arquitetura. A tela de
relacionamento PEN1 demonstra o complementar protocolo_RI1023, a tabela
Lista de Processos Relacionados e a remoção vinculada à linha. A cassette
PEN1 registra os passos 13 (GET da tela), 14 (POST de exclusão retornando 302)
e 15 (releitura 200).

A fatia implementa a leitura da barra e da tabela via html_atributos,
identificação do destino pela consulta AJAX de leitura com formulário_web,
e a confirmação (--confirmar) com escrita única:
1. Para adicionar: POST de sbmAdicionar com os campos derivados do formulário.
2. Para remover: POST com a action procedimento_excluir_relacionamento extraída
   de acaoRemover servida, com hdnInfraItemId.
3. Verificação por releitura: novo GET da tela de relacionamentos para
   comprovar que a lista foi alterada conforme o pedido.
4. Efeito incerto: se a releitura não refletir a alteração, encerra com saída 3
   sem retentativas.

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

Adotada a estratégia Web com prévia e confirmação via POST único derivado
da tela servida. A verificação é sempre realizada por releitura independente
da lista de relacionamentos. Nenhum retry automático é executado.

## Evidências verificáveis

- [Contrato de CLI](../../contracts/cli/processo-relacionar.md).
- [Evidência da operação](../../contracts/evidencias/processo-relacionar.md).
- [Teste CLI](../../tests/cli_processo_relacionar.rs).
- [Casos de tela e XML](../../src/processo_relacionar/tests.rs).
- [Testes de escrita](../../src/processo_relacionar/tests_escrita.rs).
- [Relatório de execução](../validacao-live/2026-11-05-processo-relacionar-handoff.md).

## Limitações e revisão

O parser PEN1 não prova paridade nas demais instalações, nem o número
exato de pedidos na primeira abertura de um processo sem índice. A escrita
foi verificada sinteticamente e autorizada em treinamento. Live NOT_RUN;
nenhum resíduo deixado no SEI.
