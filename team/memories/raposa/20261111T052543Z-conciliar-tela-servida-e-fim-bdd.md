---
type: AgentMemory
title: "Ablação #721: conciliar TelaServida com atribuição tipada e encerrar BDD"
description: "PR #750 reconciliada por merges verdadeiros, preservando a tipagem observada da main; gates 7/7 no SHA b86e7e20. Nova decisão proíbe novos BDD/Gherkin, exige E2E cassette-first."
agent: raposa
timestamp: "2026-11-11T05:25:43Z"
tags: [ux, ablacao, tela-servida, atributo-tipado, e2e, cassette-first, ci, cafezinho]
---

# Para retomar

**Fatos:** [PR #750](https://github.com/franklinbaldo/ies/pull/750) remove dois `#[allow(clippy::too_many_arguments)]` de prazo e atribuição pela passagem de `TelaServida` preexistente. Reconcilei os conflitos semânticos de três arquivos com a `main`, preservando `Atribuicao` tipada e testes novos. Merge verdadeiro `307222d9`; a atualização `b86e7e20` incorporou apenas memória da 🦫. No head `b86e7e20`, [CI oficial 7/7](https://github.com/franklinbaldo/ies/actions/runs/42965263234) e [rápido PASS](https://github.com/franklinbaldo/ies/actions/runs/42965248566), delta quatro arquivos (+38/-45), `behind=0` antes desta memória.

**Decisão vigente:** o [roadmap #56](https://github.com/franklinbaldo/ies/issues/56) aposenta BDD/Gherkin/Cucumber; priorizar ablações #716–#723 e E2E via cassettes originais, Rust real, anti-IES/SQLite. A [PR #758](https://github.com/franklinbaldo/ies/pull/758) é da 🐘: não duplicar sua migração. No #750 cards UnB de prazo e atribuição (33/24 interações) foram lidos, mas confronto bilateral com corpos originais e treinamento permanecem NOT_RUN; nenhuma fixture nova.

**Cafezinho:** li [#388](https://github.com/franklinbaldo/ies/issues/388), inclusive [nota da 🦫 sobre NotADirectory](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6787699174); a lição útil é preservar precisamente a causa sem prometer que a fronteira pública também a preservou.

**CONTINUAR:** sincronizar esta memória aditiva da `main` para a branch da #750 sem alterar código, obter CI no SHA final e solicitar re-review 🦅/integração 🐘/🦡; não começar outra fatia enquanto #750 estiver aberta.
