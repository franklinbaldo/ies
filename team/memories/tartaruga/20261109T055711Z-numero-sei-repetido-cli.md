---
type: AgentMemory
title: "BDD-60-05: número SEI repetido entre nós distintos"
description: "PR #540 integrada: CLI real recusa dois documentos com mesmo número na árvore do anti-IES/SQLite; CI 7/7, paridade cassette e treinamento pendentes."
agent: tartaruga
timestamp: "2026-11-09T05:57:11Z"
tags: [bdd, anti-ies, identidade, cli, navegacao, cassette]
---

# Para retomar

**Fato:** a [PR #540](https://github.com/franklinbaldo/ies/pull/540) integrou a regressão E2E do BDD-60-05, por [merge 6178bbee](https://github.com/franklinbaldo/ies/commit/6178bbee5f779474c83896ba67838a112868ee99). Dois nós de IDs diferentes com o mesmo número SEI no SQLite fazem a CLI real recusar `numero_sei_repetido_na_arvore` (exit 1), mesmo com `--ocultar-inacessiveis`, sem GET de corpo nem escrita. O [CI completo #42569728272](https://github.com/franklinbaldo/ies/actions/runs/42569728272) fechou 7/7 no head `04ab5ec`. Não houve RED funcional, apenas correções intermediárias do harness e gates.

**Limites:** [inspeção UnB/GO #42568327726](https://github.com/franklinbaldo/ies/actions/runs/42568327726) validou hashes/estrutura de 30 interações HTTP, mas não reproduziu semanticamente a duplicação nem prova que ela ocorre no SEI. Paridade independente e live são NOT_RUN; a duplicação é hipótese de teste.

**Próximo:** não reabrir a mesma fatia já integrada; procurar lacuna real ainda prevista no BDD-60-05 ou confronto independente com as observações originais. No [Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6733365319), comparei o mesmo título de duas faixas à mesma projeção com identidades distintas: inspiração, não evidência SEI.
