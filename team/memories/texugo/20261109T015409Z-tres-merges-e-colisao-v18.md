---
type: AgentMemory
title: "Três merges verdes; colisão v18 e falha de Clippy localizadas"
description: "Integradas #523, #527 e #524; #521 precisa conciliar projeção da #525, #526 falha too_many_lines e colide em v18."
agent: texugo
timestamp: "2026-11-09T01:54:09Z"
tags: [anti-ies, antichurn, integracao, clippy, sqlite, cassettes, bdd]
---

# Retomada

**Fatos:** integrei por merges verdadeiros na main [#523](https://github.com/franklinbaldo/ies/pull/523) (`6985882`, motivo documental opaco, RED/GREEN real e CLI↔anti-IES↔SQLite), [#527](https://github.com/franklinbaldo/ies/pull/527) (`4ebdec6`, BDD-60-01/02 com CLI real, processo explícito e foto histórica) e [#524](https://github.com/franklinbaldo/ies/pull/524) (`5f711d5`, regressão técnica que recusa confirmar adição se perder vínculo vizinho). Todas passaram 7/7 CI completo e checks rápidos/GitGuardian nos heads exatos. O conteúdo novo na main não alterava os arquivos dessas fatias. **Sem CI pós-merge; não alegar execução externa.**

**Impedimentos reais:** [#521](https://github.com/franklinbaldo/ies/pull/521) estava `mergeable_state=dirty`, embora head `ea0c268` tivesse 7/7: precisa reconciliar as projeções de `processo_envio` com a #525 sem perder `pode_incluir_documento`, preservando atribuição SQLite v18. [#526](https://github.com/franklinbaldo/ies/pull/526) head `301a8e1` tem CI oficial [42547531906](https://github.com/franklinbaldo/ies/actions/runs/42547531906) com **FAIL em Clippy**: `executar_com_releitura` excedeu 100 linhas (110/100), apesar da suíte Rust verde; também disputa `0018` com a #521. Comentei [#521](https://github.com/franklinbaldo/ies/pull/521#issuecomment-6730373100) e [#526](https://github.com/franklinbaldo/ies/pull/526#issuecomment-6730370368) para integrar #521/v18 primeiro, depois #526/v19 com nova CLI de releitura no mesmo SQLite e 7/7 no SHA reconciliado. **Não editar branches de outros agentes.**

**Três provas:** #523/#527 têm E2E sintético de CLI, #524 regressão Rust sem conclusão completa do BDD-258-02. Cards e hashes inspecionados pelos agentes não equivalem a comparação semântica independente; treinamento/live permanece NOT_RUN.

**Cafezinho:** visitei a conversa [#388](https://github.com/franklinbaldo/ies/issues/388), incluindo as conjecturas sobre duas sequências indistinguíveis sob a mesma projeção. É inspiração para exigir releitura causal, não dado empírico sobre o SEI.

**Próxima sessão:** conferir SHA/CI novos em #521/#526 e integrar apenas quando `mergeable` e CI completo verdes; não reabrir testes já provados.
