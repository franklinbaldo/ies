---
type: AgentMemory
title: "Vínculo bilateral não deve ocupar duas arestas"
description: "RED/GREEN no runner demonstrou duplicação reversa silenciosa; o modelo agora impede registro inverso redundante, mas a PR aguarda SQLite v14."
agent: castor
timestamp: "2026-11-08T20:03:39Z"
tags: [rust, sqlite, anti-ies, bdd, causalidade]
---

# Para a próxima rodada

Na [PR #491](https://github.com/franklinbaldo/ies/pull/491), dois registros `10→22` e `22→10` viravam duas linhas distintas, embora `listar` deduplicasse. O [RED/GREEN real #42518016747](https://github.com/franklinbaldo/ies/actions/runs/42518016747) reproduziu **2 em vez de 1** no SHA anterior e aprovou **1** no corrigido, com rustfmt. O workflow temporário saiu do diff. Isso é uma regressão técnica do estado sintético; **não** comprova cassettes ou SEI real.

**Retomar:** #487/v13 entrou; #489/v14 ainda estava Draft na consulta. Só depois de #489 integrada incorporar `main` por merge verdadeiro, registrar SQL v12→v13→v14→v15 e conferir CI completo no SHA final da #491. Os BDD-258-02/03/04 seguem pendentes; não habilitar escrita externa sem a validação prevista.

No [Cafezinho](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6726392005), o 🦡 comparou ordem de dobras a ordem de pedais no áudio: duas sequências podem produzir vestígios indistinguíveis. Inspiração ficcional para exigir observação independente, não evidência técnica.
