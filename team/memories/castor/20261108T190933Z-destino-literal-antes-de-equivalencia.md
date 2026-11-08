---
type: AgentMemory
title: "Destino literal prevalece sobre equivalência ambígua"
description: "RED comprovou falso DestinoAmbiguo com dois NUPs equivalentes; correção respeita identidade exata antes da normalização."
agent: castor
timestamp: "2026-11-08T19:09:33Z"
tags: [bdd, rust, sqlite, anti-ies, causalidade]
---

# Encaixe para a próxima rodada

Na [PR #491](https://github.com/franklinbaldo/ies/pull/491), sincronizei `main@97a05b3` por merge verdadeiro `d24c989`. O [RED oficial #42513072552](https://github.com/franklinbaldo/ies/actions/runs/42513072552) mostrou **1 FAIL em 12 testes `iesei`**: `identificar("AB-C")` escolhia `DestinoAmbiguo` ao encontrar também `A-BC`, embora o NUP literal fosse único. O modelo agora prioriza literal único e só usa equivalência sem pontuação na ausência dele, recusando múltiplos equivalentes. Head `5584bbf`: [CI rápido PASS](https://github.com/franklinbaldo/ies/actions/runs/42513301287), [CI completo #42513320516](https://github.com/franklinbaldo/ies/actions/runs/42513320516) com Rust/OKF PASS e replay GO pendente na última consulta; **não antecipar 7/7**.

A prévia IES ↔ anti-IES permanece sintética; card PEN1 já inspecionado, mas confronto semântico e live **não executados nesta rodada**. A PR segue Draft até integrar #487 (SQLite v13) e #489 (v14), então conciliar a nossa v15 por merge e CI novo. Não repetir essa correção se já estiver verde. Hipótese para próximo teste tipado, se relevante ao BDD-258-03: registrar 10→22 e depois 22→10 pode guardar duas linhas direcionais, embora a listagem deduplique; investigar antes de habilitar remoção bilateral.