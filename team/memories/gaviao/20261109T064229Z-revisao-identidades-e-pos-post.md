---
type: AgentMemory
title: "Três heads verdes: identidade contraditória e prévia após POST"
description: "PRs #542/#543/#544 revisadas com CI 7/7 nos respectivos SHAs; diferenciação entre regressão técnica, CLI após POST local e paridade de cassettes. #526 continua bloqueada por v18."
agent: gaviao
timestamp: "2026-11-09T06:42:29Z"
tags: [revisao, anti-ies, bdd, cassettes, ci, identidade, sqlite, cafezinho]
---

# Para a próxima rodada

**Fatos:** publiquei pareceres nos heads [#542](https://github.com/franklinbaldo/ies/pull/542#issuecomment-6733970934) `114b7c4` (mesmo id AJAX e NUP distinto na tabela deve recusar, não `SemEfeito`), [#543](https://github.com/franklinbaldo/ies/pull/543#issuecomment-6733971369) `76f9dd4` (PDF em árvore mista não pode vencer seleção `id_documento`×`id_protocolo` divergente) e [#544](https://github.com/franklinbaldo/ies/pull/544#issuecomment-6733971779) `6cdffb4` (POST local altera SQLite, CLI real vê estado na prévia seguinte). Todos os três possuem **CI oficial 7/7 no SHA exato**, diffs lidos e `mergeable_state=clean`. Comparação com `main`: atrás respectivamente 4, 4 e 1 commits, **todos memórias OKF**, sem sobreposição funcional observada. Se sincronizarem, revalidar novo SHA. A #544 é incremento de teste, não confirmação `--confirmar`.

**Pendente antigo:** [#526](https://github.com/franklinbaldo/ies/pull/526) ainda `5188493`, `mergeable_state=dirty`, agora 215 commits atrás; mantém criação em migração `0018` concorrente com atribuição da `main`. Exigir merge verdadeiro, criação em v19, preservação da navegação e CI 7/7 no SHA reconciliado; não refazer E2E sintético.

**Três provas:** testes sintéticos e integridade de ZIPs são distintos de confronto semântico independente de ambas as implementações com cassettes; treinamento ainda NOT_RUN nessas fatias. Não vetar por ausência da prova externa.

**Cafezinho:** visitei [#388](https://github.com/franklinbaldo/ies/issues/388), aberto com 51 comentários. A conversa sobre duas testemunhas atribuírem identidades diferentes ao mesmo pino ajuda a lembrar a necessidade de coerência, mas a metáfora da prensa e o conflito AJAX são ficção/hipótese sintética, não observação no SEI PEN1.
