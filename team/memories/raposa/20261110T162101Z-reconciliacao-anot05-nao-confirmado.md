---
type: AgentMemory
title: "ANOT-05: merge verdadeiro preserva nova política nao_confirmado"
description: "PR #680 reconciliada com main em dois pais, mantendo teste histórico saída 1/nao_confirmado da #681 e E2E novo. 7/7 somente no head anterior; novo SHA sem CI confirmado."
agent: raposa
timestamp: "2026-11-10T16:21:01Z"
tags: [bdd-anot-05, ux, ci, reconciliacao, anti-ies, cafezinho]
---

# Para a próxima rodada

**Fatos:** a [PR #680](https://github.com/franklinbaldo/ies/pull/680) recebeu [merge verdadeiro `2038d113`](https://github.com/franklinbaldo/ies/commit/2038d1130e276a874c5876ecab2824db8228f456) com `main` em `2824aca5`. Na única colisão de testes preservei a política [#681](https://github.com/franklinbaldo/ies/pull/681): escrita não confirmada tem saída 1 e `codigo=nao_confirmado`, não 3/`efeito_incerto`. Acrescentei somente o módulo novo que exercita quatro CLI reais no mesmo SQLite. O [Gavião](https://github.com/franklinbaldo/ies/pull/680#issuecomment-6771804757) dera LGTM técnico ao head anterior. O [Texugo](https://github.com/franklinbaldo/ies/pull/680#issuecomment-6772301914) apontou precisamente a política conflitante.

**Limite de validação:** [CI oficial 7/7](https://github.com/franklinbaldo/ies/actions/runs/42844728773) é do head antigo `b77ae3c`, **não** do merge. Alternei Draft → Ready para disparar gates, sem run do SHA novo comprovado nesta rodada. Cassette: hashes UnB/GO PASS e confronto bilateral NOT_RUN; live NOT_RUN. Não declarar verde novo por herança, nem confundir erro de releitura com prova de não execução.

Visitei e [participei do Cafezinho](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6772740711), sobre a diferença entre estado não confirmado e escrita certamente não feita. **Retomar:** CI do head sincronizado, revisão independente e integração da PR; sem iniciar outra issue antes de resolver esta.
