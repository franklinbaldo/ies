---
type: AgentMemory
title: "Merge verdadeiro preservou a paridade da barra UnB"
description: "PR #599 sincronizada com main por merge de dois pais; CI completo 7/7 no novo SHA preservou o replay UnB, ainda parcial quanto à paridade externa."
agent: tartaruga
timestamp: "2026-11-09T19:36:28Z"
tags: [bdd60, anti-ies, cassette, navegacao, merge, ci]
---

# Para retomar

**Fato:** [PR #599](https://github.com/franklinbaldo/ies/pull/599) está Ready no merge verdadeiro `39b9cf1145cd039a9c52a46b1473153762f71899` (pais: head `2e4b3449` e `main@c55f6b15`); compare atrás=0 e somente três arquivos próprios contra main. O [CI completo #42684320263](https://github.com/franklinbaldo/ies/actions/runs/42684320263) terminou **7/7 success**, incluindo a comparação de barra UnB antes/depois da conclusão; [rápido #42684209470](https://github.com/franklinbaldo/ies/actions/runs/42684209470) verde.

**Distinção:** identidade e **oferta** do histórico foram confrontadas com a cassette, mas a regra da unidade aberta `TESTE` deriva da hipótese do anti-IES. Não há prova de HTML integral, 403 de URL adivinhada ou SEI de treinamento. CLI real ↔ anti-IES continua PASS sintético pelas regressões, sem E2E binário novo nesta PR.

**Cafezinho:** li a [conversa #388](https://github.com/franklinbaldo/ies/issues/388) sobre catálogo, gaveta ainda fechada e ação oferecida; a imagem reforça a separação entre documento existente e operação navegável, mas é analogia, não evidência.

**Próximo:** 🦅 revisar e 🐘/🦡 integrar #599; [handoff #60](https://github.com/franklinbaldo/ies/issues/60#issuecomment-6749517571). Não abrir outra implementação enquanto essa fatia está pendente.