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

## Contexto

No GO SEI 5.0, `documento_cancelar` apareceu na barra de um documento assinado depois de tramitar o processo. Na UFRJ e no PEN3 a ação também foi observada; na UnB 4.0.9 não apareceu para o usuário testado. A observação do Todos @323eb01 mostra `frmDocumentoCancelar` com motivo e botão Salvar, retorno 302 à `arvore_visualizar` e evidência na árvore por ícone `documento_cancelado`.

## Alternativas consideradas

1. **Web, formulário publicado e verificação por árvore** (escolhida): não inventa ação, deriva apenas os campos servidos e exige mudança de estado observável.
2. **REST/WSSEI**: não foi demonstrada uma ação de cancelamento equivalente nas instalações.
3. **Montar o formulário sem abrir a tela**: menos GET, porém perde o `action` assinado, a codificação e os controles realmente servidos.

## Comparação

| Critério | Web servida | REST | Formulário reconstruído |
| --- | --- | --- | --- |
| Ação observada na barra | Sim (GO/UFRJ/PEN3) | Não demonstrada | Link observado, corpo não |
| Endereço e campos assinados | Da resposta do SEI | Não verificado | Construídos pelo cliente |
| Prévia a frio com --processo | 4 GET | Indeterminado | 3 GET |
| Confirmação + verificação | 5 GET + 1 POST | Indeterminado | 4 GET + 1 POST, não seguro |
| Escrita não confirmada sem retry | Sim | Indeterminado | Indeterminado |

## Decisão

A alternativa escolhida **não presume disponibilidade da ação**: exige âncora única na barra do documento identificado. Não monta URLs assinadas, não usa REST e não executa JavaScript. O formulário é derivado pelo `formulario_web`, com efeito classificado como mudança do estado do processo. Nenhuma escrita acontece durante a prévia.

O POST único é consumido, sem retries. O 302 é analisado como dado (nunca seguido); o cancelamento só é provado pela árvore relida, conferida quanto ao número/id, ícone e motivo. Se a leitura não decide, saída 1: não repetir. Com `--processo` a frio, prévia custa 4 GET e confirmação 5 GET + 1 POST.

## Evidências verificáveis

- [Contrato](../../contracts/cli/documento-cancelar.md) e [evidência da operação](../../contracts/evidencias/documento-cancelar.md).
- [Testes da superfície CLI](../../tests/cli_documento_cancelar.rs), além dos sintéticos em `src/documento_cancelar.rs` e `src/documento_cancelar/tests_fluxo.rs`.
- Acervo GO/UFRJ/PEN3 e índice UnB: links na evidência. Dados originais do Todos @323eb01, não arquitetura transplantada.
- CI da [PR #282](https://github.com/franklinbaldo/ies/pull/282): a validação depende do head atual, não da existência deste documento.

## Limitações e revisão

A escrita live e o replay do acervo ainda são **NOT_RUN** (0 requisições reais, sem resíduos no SEI). A ordem para um ensaio futuro é: somente instalação de **treinamento** com a ação oferecida, prévia → um único `--confirmar` → leitura independente. Na UnB não se deve tentar forçar ação que a barra não publique. Credenciais de treinamento não foram disponibilizadas a este sandbox e não houve tentativa de escrita.

O parser foi derivado de formas observadas pelo Todos; novos estilos de barra, formulários ou retornos exigem novos casos RED, nunca fallback por heurística.
