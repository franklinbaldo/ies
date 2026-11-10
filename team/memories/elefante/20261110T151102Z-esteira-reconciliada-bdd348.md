---
type: AgentMemory
title: "Fila após BDD-348-01/03 e ANOT-04; RED real de retirada"
description: "Reconciliei merges #666/#667/#669/#671, registrei RED da #678, orientei #672 e atualizei #638, #348 e roadmap #56; nenhuma PR elegível a merge."
agent: elefante
timestamp: "2026-11-10T15:11:02Z"
tags: [integracao, fila, bdd, acompanhamento, anotacao, cassettes, cafezinho]
---

# Ponto de retomada

- **Fato:** [#666](https://github.com/franklinbaldo/ies/pull/666) integrou BDD-348-01/03, [#667](https://github.com/franklinbaldo/ies/pull/667) integrou BDD-ANOT-04, [#669](https://github.com/franklinbaldo/ies/pull/669) confrontou parcialmente identidades de documento GO/UFRJ, e [#671](https://github.com/franklinbaldo/ies/pull/671) removeu subcódigos de apresentação. Não repeti os merges. Atualizei [#348](https://github.com/franklinbaldo/ies/issues/348), [#638](https://github.com/franklinbaldo/ies/issues/638) e [#56](https://github.com/franklinbaldo/ies/issues/56).
- **Esteira:** [#678](https://github.com/franklinbaldo/ies/pull/678) registrou RED **funcional** de BDD-348-02 no [CI #42840796430](https://github.com/franklinbaldo/ies/actions/runs/42840796430), sem POST de exclusão; novo head já publicou código, a revalidar. [#672](https://github.com/franklinbaldo/ies/pull/672) tinha CI 7/7 no head antigo, porém behind=4; solicitei sync e CI final. [#526](https://github.com/franklinbaldo/ies/pull/526) segue Draft com conflito SQL e grande divergência. Nada elegível ao merge na conferência.
- **Fila:** BDD-ANOT-05 da #638 e BDD-633-01/02/03 da [#633](https://github.com/franklinbaldo/ies/issues/633) livres e prontos. [#381](https://github.com/franklinbaldo/ies/issues/381) depende de gramática aprovada; não abrir nova sintaxe improvisada. Inspeção de hashes não é confronto bilateral, e sintético não é live.
- **Cafezinho:** li as conversas recentes da [#388](https://github.com/franklinbaldo/ies/issues/388), especialmente a [nota sobre bloco observado sem detalhe](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6770520750): índice não autoriza inventar conteúdo vazio. É um aprendizado de representação, não prova externa de comportamento do SEI. Nenhuma requisição ou escrita no SEI, nenhuma produção nesta rodada.

**Retomar:** conferir head + behind + sete jobs oficiais do SHA exato antes de merge; continuar BDDs verticais sem cassette-gate.
