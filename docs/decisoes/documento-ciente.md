---
type: ImplementationDecision
title: Ciência documental com GET único e lista CD do próprio nó
operation: documento.ciente
implementation: partial
status: adopted-for-scope
scope: prévia, GET único e prova por lista de ciências do documento
selected_strategy: tela-do-no-get-unico-atividade-lista-documento
alternatives: tela-do-no-get-unico-atividade-lista-documento,aceitar-resposta-http,listar-ciencias-do-processo,rest-wssei
evidence_level: oraculo-todos-unb-e-testes-sinteticos
live_validation: not-run
---

# Decisão de ciência documental — #271

## Contexto

O comando trabalha pelo SEI Web, aproveitando a abertura de documento na
árvore conferida. A ação de documento é distinta de `procedimento_ciencia`:
a barra `cienciaDocumento();`, o GET `documento_ciencia` e o marcador
`documento_visualizar_ciencias=1` são evidências do oráculo Todos
`test_documento_ciente_010.py` @323eb01 (UnB 4.0.9).
Na tela do nó há função 4.0 ou variável 4.1/5.0. Inspecionar, não
executar JavaScript. URLs precisam ser assinadas, da origem permitida,
vinculadas ao documento e conferidas antes do GET.

## Alternativas consideradas

1. **GET observado + Lista CD do documento relida** (escolhida): prova pelo marcador exato de atividade.
2. **Aceite HTTP apenas**: não comprova registro persistido.
3. **Lista de Ciências do processo**: não distingue o documento.
4. **REST/WSSEI**: não há evidência para selecionar.

## Comparação

| Critério | Lista CD relida | Só aceite HTTP | Lista do processo | REST |
| --- | --- | --- | --- | --- |
| Prova vinculada ao documento | sim | não | não | não observado |
| Sem retry e URL servida | sim | sim | sim | não observado |
| Identifica a atividade nova | sim | não suficiente | não | não observado |

## Decisão

**Escolha:** prévia só da árvore; confirmação por um único GET;
depois reabrir a árvore e exigir a ação de Lista de Ciências do
`CD<id>` do documento, com processo idêntico e a atividade exata do
aceite. Se faltar prova, saída 1. A resposta HTTP, a Lista de Ciências
do processo e o histórico genérico não provam. O backend REST/WSSEI
não foi observado e não é selecionado.

O GET é escrita; falha de rede, validação ou 302 não disparam nova
tentativa. O resultado do GET é mantido como causa tipada dentro da falha
de escrita não confirmada; a prova é sempre a releitura, sem presumir que HTTP
403, validação da Infra ou timeout significam não aplicação. A URL
de <script> aninhado em <template> também é recusada, independentemente
de o parser expor esse nó. Não restaurar telas. O código reutiliza `formulario_web`
quando aplicável, a abertura de documento, o transporte/contador
compartilhados, leitores HTML e os erros tipados.

## Decisão de segurança da URL mutadora

A publicação do botão na barra não basta: a URL do GET mutador precisa
estar em código ativo da tela do próprio nó. O leitor reconhece apenas
a atribuição estática no topo do script (4.1/5.0) ou a forma observada
da função com destino literal (4.0). Não interpreta JavaScript: ignora
comentários, templates, strings e ramos condicionais, inclusive quando
a forma textual da URL é idêntica à legítima. Na dúvida, recusa
antes do GET mutador, ainda que haja botão. A regressão está em
`src/ciencia/tests_documento.rs` e o RED na
[observação live](../validacao-live/2026-11-06-documento-ciente.md).

## Limitações e revisão

**Trade-offs:** mais uma leitura para prova e uma para a lista;
pode haver escrita não confirmada e o usuário deve consultar o SEI antes de
repetir. A prévia não modifica o documento. Uma instalação não
comprova todas as demais.

## Evidências verificáveis

[Contrato](../../contracts/cli/documento-ciente.md) ·
[matriz de evidências](../../contracts/evidencias/documento-ciente.md) ·
[teste da CLI](../../tests/cli_documento_ciente.rs) ·
[observação](../validacao-live/2026-11-06-documento-ciente.md).
Revisão pendente: replay do acervo, comportamento da segunda ciência e
prévia/confirmação/releitura em duas instalações de treinamento.


## Verificação da tabela dentro de `<template>` (#271)

Mesmo depois de conferir `CD<id>`, a prova precisa vir de uma tabela
realmente publicada. Um `table#tblAndamentos` sob `<template>` é
encontrável pelo parser, mas está inerte no DOM do navegador; não prova
que a ciência foi gravada. Optou-se por **recusar apenas quando o template
é ancestral ou descendente da tabela de prova**, sem invalidar templates
sem relação com ela. Teste RED (antes do código) e fluxo de escrita não confirmada
no relatório live; não há JavaScript executado, retry ou GET extra.

## Decisão adicional: prova HTML inerte e ciência repetida (06/11/2026)

**Alternativa escolhida:** tratar `<noscript>` como contêiner de prova
não confiável, tal qual `<template>`, somente quando ancestral ou
descendente da tabela `tblAndamentos`. Com scripts habilitados no SEI,
uma linha inerte achada pelo parser não demonstra execução nem registro.
É preferível sair com escrita não confirmada após uma leitura a aceitar um
falso sucesso. Não interpretar nem executar JavaScript.

Para a segunda ciência, manter a diferença observada no Todos:
processo = repetição no mesmo dia; documento = repetição no documento,
sem limite diário. A validação `txaInfraValidacao` de HTTP 200 é uma
recusa, não um aceite com atividade, e exige a mesma reconciliação sem
retry; o IES ainda não afirma equivalência live ou idempotência.

RED sintético da prova em `<noscript>`:
[run 42294900770](https://github.com/franklinbaldo/ies/actions/runs/42294900770),
2 FAIL, 1.770 PASS, 22 ignores anteriores. Testes acrescentados em
`src/ciencia/tests.rs`, `src/ciencia/tests_fluxo.rs` e
`src/ciencia/tests_documento/efeitos_incertos.rs`. CI de GREEN deve ser
conferido na PR #337, sempre no head exato.

## Recusa de scripts em `noscript` e replay específico

O leitor de aceite, além de recusar provas de tabela inerte, rejeita
`atualizarArvore` quando o script está em `<noscript>` e o SEI exige
JavaScript ativo. A árvore deve ser relida mesmo se a resposta do GET
mutador for ilegível; não usar ciência anterior como sucesso.

O [RED 42302428489](https://github.com/franklinbaldo/ies/actions/runs/42302428489)
provou duas falhas antes do código: leitor retornava `Ok(id)` para
script inerte e o fluxo tentava ler a lista sem aceite comprovável
(1.775 PASS, 2 FAIL, 22 ignorados anteriores).

Para confrontar as cassettes, o teste Rust `ciencia::acervo::replay_da_ciencia_no_acervo`
lê exclusivamente os 14 cards das sete instalações; reutiliza o
conferidor de ZIP/cassette e os leitores `atividade_do_aceite` e
`atividade_na_lista`, sem executar GET nem JavaScript. A classificação
é somente das **respostas gravadas**, não prova de escrita atual.

## Lacuna observada no replay dedicado (#271)

A conferência offline de 14 cards passou no
[run 42303368527](https://github.com/franklinbaldo/ies/actions/runs/42303368527),
mas 28 respostas de `procedimento_ciencia`/`documento_ciencia` não
forneceram atividade reconhecida pelo leitor atual; 14 listas ficaram
sem `id_atividade` verificável. Isso é **evidência de lacuna de
conformidade**, não prova de falha do SEI. Próxima investigação:
confrontar, de forma saneada, respostas preparatórias e mutadoras
das capturas e seus HTMLs; manter saída 1 quando não houver prova.
O teste offline não substitui prévia nem escrita live.
