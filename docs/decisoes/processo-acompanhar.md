---
type: ImplementationDecision
title: Acompanhamento Especial por leitura estrita da barra e do cadastro
description: Decisão parcial da prévia Web do acompanhamento de processos
operation: processo.acompanhar
implementation: partial
status: adopted-for-scope
selected_strategy: web-barra-cadastro-tabela
alternatives: web-barra-cadastro-tabela,rest-acompanhamento,simular-estado
evidence_level: todos-unb-e-cassettes-indexadas
live_validation: not-run
---

# Acompanhamento Especial pela interface Web (Refs #254)

## Evidência observada

O Todos `src/todos/acompanhamento.py` e
`tests/test_processo_acompanhar_010.py` @323eb01 registram as
formas da UnB (15–16/10/2026): link
`acompanhamento_gerenciar` na barra; 302 ao cadastro quando
não há linhas; tabela `Tabela de Acompanhamentos.` quando há;
`btnAdicionar` para o cadastro; formulário
`frmAcompanhamentoCadastro` com select
`selGrupoAcompanhamento`, observação `txaObservacao` e
`hdnIdProtocolo`. Uma unidade oferece grupos diferentes da outra.
O índice de cassette
[`processo-acompanhar-unb`](../../cassettes/unb/processo-acompanhar-unb--4e19a6146bc4.md)
registra 45 interações (39 GET, 6 POST); o índice, sozinho, não
comprova o conteúdo dos formulários nem paridade da implementação.

## Alternativas e decisão

1. **Web por link e forma servidos (escolhida):** reutiliza a sessão,
   valida a URL assinada publicada, não executa JS, não segue redirects,
   recusa lista incompleta e não adivinha grupos. Os caminhos próprios
   do índice reduzem as requisições de abertura.
2. **REST:** não é a prioridade do IES e exige contrato/evidência de
   rota habilitada; rejeitada neste recorte.
3. **Inferir por ausência de tabela ou montar endpoint:** arriscaria
   declarar vazio num HTML truncado e usar URL não oferecida; rejeitada.

## Escopo, trade-offs e revisão

Esta primeira fatia implementa a **prévia real** (GETs, sem escrita) e
a seleção/reconciliação puras com testes sintéticos. Não afirma
cobertura de todas as versões; falta replay do acervo e leitura ao vivo
de UnB e segunda instalação. O POST único, a classificação da resposta,
a verificação por releitura e a remoção permanecem bloqueados atrás
de `--confirmar`, recusado antes de tocar a sessão. A próxima decisão
deve testar essas transições com transporte injetado e validar orçamento
exato por instalação. O contrato é [processo.acompanhar](../../contracts/cli/processo-acompanhar.md).
