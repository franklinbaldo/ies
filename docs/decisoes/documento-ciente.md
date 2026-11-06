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

O comando trabalha pelo SEI Web, aproveitando a abertura de documento na
árvore conferida. A ação de documento é distinta de `procedimento_ciencia`:
a barra `cienciaDocumento();`, o GET `documento_ciencia` e o marcador
`documento_visualizar_ciencias=1` são evidências do oráculo Todos
`test_documento_ciente_010.py` @323eb01 (UnB 4.0.9).
Na tela do nó há função 4.0 ou variável 4.1/5.0. Inspecionar, não
executar JavaScript. URLs precisam ser assinadas, da origem permitida,
vinculadas ao documento e conferidas antes do GET.

**Escolha:** prévia só da árvore; confirmação por um único GET;
depois reabrir a árvore e exigir a ação de Lista de Ciências do
`CD<id>` do documento, com processo idêntico e a atividade exata do
aceite. Se faltar prova, saída 3. A resposta HTTP, a Lista de Ciências
do processo e o histórico genérico não provam. O backend REST/WSSEI
não foi observado e não é selecionado.

O GET é escrita; falha de rede, validação ou 302 não disparam nova
tentativa. Não restaurar telas. O código reutiliza `formulario_web`
quando aplicável, a abertura de documento, o transporte/contador
compartilhados, leitores HTML e os erros tipados.

**Trade-offs:** mais uma leitura para prova e uma para a lista;
pode haver efeito incerto e o usuário deve consultar o SEI antes de
repetir. A prévia não modifica o documento. Uma instalação não
comprova todas as demais.

[Contrato](../../contracts/cli/documento-ciente.md) ·
[observação](../validacao-live/2026-11-06-documento-ciente.md).
Revisão pendente: replay do acervo, comportamento da segunda ciência e
prévia/confirmação/releitura em duas instalações de treinamento.
