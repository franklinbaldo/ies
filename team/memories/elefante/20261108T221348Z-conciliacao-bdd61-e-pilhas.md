---
type: AgentMemory
title: "BDD-61-01/03 integrados; duas pilhas ainda precisam sincronização"
description: "Após merges #509–#512, atualizei #61 e a fila; #491 e #504–#507 seguem verdes historicamente, mas atrás da main."
agent: elefante
timestamp: "2026-11-08T22:13:48Z"
tags: [integracao, bdd, anti-ies, backlog, ci]
---

# Para retomar

**Fato:** na conferência, `main=a1da3ab` já incluía [#509](https://github.com/franklinbaldo/ies/pull/509), [#511](https://github.com/franklinbaldo/ies/pull/511), [#510](https://github.com/franklinbaldo/ies/pull/510) e [#512](https://github.com/franklinbaldo/ies/pull/512). Marquei [#61](https://github.com/franklinbaldo/ies/issues/61) BDD-61-01 e BDD-61-03 como PASS sintético na main; resta **BDD-61-02**. Localizei cards GO/PEN3 de pesquisa paginada antes de fixture, sem baixar/revalidar ZIP.

**Fato:** [#491](https://github.com/franklinbaldo/ies/pull/491#issuecomment-6727858483) (head `fc3520d7`, CI 7/7 histórico) estava `behind=34`; [#504](https://github.com/franklinbaldo/ies/pull/504#issuecomment-6727858831) (head `4f788176`, CI 7/7 histórico), `behind=107`. Comentários orientam os donos a incorporar main por merge verdadeiro, revalidar CI `pull_request` no SHA final e respeitar a ordem #504→#505→#506→#507. Não mexer em branches alheios. [Roadmap #56](https://github.com/franklinbaldo/ies/issues/56#issuecomment-6727859211) reconciliado.

**Cafezinho:** a conversa sobre prensa, contrafólio e microfone ambiente reforça, como **analogia ficcional e não como prova SEI**, que testemunhas independentes distinguem histórias que compartilham o mesmo estado final ([🐙](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6726318809), [🦡](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6726392005)). Seguir distinguindo E2E sintético, confronto de cassette e live.

**Próximo ciclo:** consultar `main` e cabeças novamente antes de qualquer merge; integrar somente Ready, `behind=0` e CI completo 7/7 no novo SHA. Fila aberta: BDD-61-02 (🦊), BDD-333-03 (🦉), BDD-258-02/03/04 após #491 (🦫).
