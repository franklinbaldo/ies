---
type: AgentMemory
title: "GO e UFRJ: confronto de seleção executado; integração depende do head novo"
description: "Os dois testes de seleção documental GO/UFRJ passaram na PR #669, mas após merge verdadeiro da main o SHA corrente exige novo CI oficial."
agent: tartaruga
timestamp: "2026-11-10T14:40:51Z"
tags: [bdd60, cassette-first, go, ufrj, navegacao, ci, sincronizacao]
---

# Para retomar

**Fato:** na [PR #669](https://github.com/franklinbaldo/ies/pull/669), o [CI completo #42826826418](https://github.com/franklinbaldo/ies/actions/runs/42826826418) passou 7/7 no head `10309c3`. O log do job `replay unidade.retornos` demonstra **GO ... ok** e **UFRJ ... ok**; a [inspeção #42825811770](https://github.com/franklinbaldo/ies/actions/runs/42825811770) confirma, em cada card, 15 interações e SHA-1/SHA-256 corretos. São confrontos **parciais** independentes de identidade/seleção/árvore; os 404/405 são hipóteses sintéticas. Não há nova prova de CLI binária nessa fatia, nem treinamento/REA.

**Sincronização:** a branch havia divergido 26 commits; os três arquivos alterados estavam idênticos na `main` e no ancestral, permitindo construir um merge genuíno de dois pais [`c964163`](https://github.com/franklinbaldo/ies/commit/c96416352f10f9fa66d8e478c777a0a141f5a2df), sem rebase ou force-push. A comparação mostrou `behind=0` e os mesmos três arquivos no diff. **A validação anterior não certifica esse commit.** Depois desta memória na `main`, conferir `behind` de novo e acionar o CI oficial (Draft → Ready) no SHA final.

**Cafezinho:** visitei a [conversa #388](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6770260457); uma prova antiga pode seguir verdadeira, mas não comprova automaticamente a base nova. Próxima rodada deve retomar #669 antes de assumir outra issue.
