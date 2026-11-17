---
type: AgentMemory
title: "BDD-258-05 integrado; duas PRs aguardam correções pequenas"
description: "A #488 entrou com sete gates; #486 falha Clippy e #487 falha releitura por proxy, com ordem de migração v12→v13."
agent: elefante
timestamp: "2026-11-08T17:11:43.726Z"
tags: [integracao, bdd, anti-ies, sqlite, ci]
---

# Retomada

Fato: a [#488](https://github.com/franklinbaldo/ies/pull/488) foi integrada na main (`5750a79`), BDD-258-05, com CI oficial 7/7. A issue #258 continua aberta pelos cenários 02/03/04.

Fato: a [#486](https://github.com/franklinbaldo/ies/pull/486) teve Clippy FAIL por `too_many_lines` no teste PDF; a [#487](https://github.com/franklinbaldo/ies/pull/487) teve BDD-273-02 FAIL porque a CLI retornou efeito incerto / proxy inacessível. Há colisão de migração 0012: primeiro PDF v12 (#486), depois cancelamento v13 (#487). Ambas precisam sincronizar com a main e passar CI 7/7 no head final; instruções já comentadas nas PRs e no [roadmap #56](https://github.com/franklinbaldo/ies/issues/56).

Lembrança do [Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6724428855), como **inspiração, não evidência técnica**: duas sequências diferentes podem deixar o mesmo estado final. Para distinguir causalidade, procurar observação que separe as histórias (antes/depois, confirmação e releitura independente); a metáfora da prensa reforça por que saída sintética não prova fidelidade de cassette.
