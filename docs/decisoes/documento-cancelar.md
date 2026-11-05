---
type: ImplementationDecision
title: Cancelamento de documento pela ação assinada da barra e verificação por leitura
operation: documento.cancelar
implementation: partial
status: adopted-for-scope
scope: Prévia sem POST, um único POST frmDocumentoCancelar, reconciliação pela árvore nova
selected_strategy: web-barra-formulario-post-unico-arvore
alternatives: web-barra-formulario-post-unico-arvore,rest-wssei,formulario-reconstruido
evidence_level: todos-323eb01-e-testes-sinteticos
live_validation: not-run
---

# Documento cancelar — decisão #273

## Contexto e escolha

No GO SEI 5.0, o `documento_cancelar` só apareceu na barra de um documento assinado depois de tramitar o processo. Na UFRJ e no PEN3 a ação também foi observada; na UnB 4.0.9 não apareceu para o usuário testado. A observação do Todos @323eb01 mostra `frmDocumentoCancelar` com motivo e botão Salvar, retorno 302 à `arvore_visualizar` e evidência na árvore por ícone `documento_cancelado` e dica do motivo.

A alternativa escolhida **não presume disponibilidade da ação**: exige âncora única na barra do documento identificado. Não monta URLs assinadas, não usa REST e não executa JavaScript. O formulário é derivado pelo `formulario_web`, com efeito explicitamente classificado como mudança do estado do processo. Nenhuma escrita acontece durante a prévia.

## Reconciliação, segurança e custos

O POST único é consumido, sem retries. O 302 é analisado como dado (nunca seguido); o cancelamento só é provado pela árvore relida, conferida quanto ao número/id, ícone e motivo. Se a leitura não decide, saída 3: não repetir. Com `--processo` a frio, prévia custa 4 GET e confirmação 5 GET + 1 POST. Nenhum URL, cookie ou hash aparece no JSON.

A alternativa REST não tem evidência de operação equivalente. Reconstruir o formulário ao invés de usar a tela servida violaria a política de evidência do IES.

## Evidência disponível e próxima validação

Contrato `contracts/cli/documento-cancelar.md`; casos sintéticos derivados de Todos @323eb01 em `src/documento_cancelar.rs` e `src/documento_cancelar/tests_fluxo.rs`. Replay do acervo e validação live permanecem **NOT_RUN**, 0 requisições reais, nenhum resíduo. Antes de qualquer escrita live, usar apenas instalação de treinamento, prévia, **um único** `--confirmar`, e leitura independente de verificação. Não testar escrita na UnB sem ação observada.
