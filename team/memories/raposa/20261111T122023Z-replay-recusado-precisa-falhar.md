---
type: AgentMemory
title: "Replay de unidade trocar não pode ficar verde com Recusado"
description: "Na PR #781, replay de ZIPs agora falha em leituras recusadas e requer transição observada; aguardar gate da #780 e CI no HEAD final."
agent: raposa
timestamp: "2026-11-11T12:20:23Z"
tags: [ux, unidade-trocar, acervo, replay, ci, cassette-first, cafezinho]
---

# Para retomar

**Fato:** na [PR #781](https://github.com/franklinbaldo/ies/pull/781), após sincronizar a branch com a `main`, vi que `replay_de_unidade_trocar_no_acervo` só publicava `Resultado::Recusado`, sem falhar. Corrigi no [commit `4e060a4b`](https://github.com/franklinbaldo/ies/commit/4e060a4b14485d429e09b80ed487a21298c694e0): agora exige acervo não vazio, transição original com Controle reconhecido e zero recusas. A ablação anterior de uma supressão Clippy permanece.

**Pendente comprovável:** a [#780](https://github.com/franklinbaldo/ies/pull/780) introduz replay oficial no Actions; o novo HEAD da #781 **não foi testado**. Deixei a PR em Draft, sem alegar equivalência bilateral ou live. Depois da integração da #780, reconciliar `main`, executar o replay original no HEAD final, conferir recusas e reabrir revisão 🦅.

**Cafezinho:** [conversa da rodada](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6795636384) sobre distinguir tabela informativa de teste que realmente falha. A tabela mostra a evidência; a asserção decide o gate.
