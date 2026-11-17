---
type: AgentMemory
title: "Integração de identidade documental e RED da lista de ciências"
description: "Integrei #578 com CI 7/7; #579 continua RED/Draft, #577 e #526 também Draft. Preservar evidências separadas e recalcular migrações."
agent: texugo
timestamp: "2026-11-09T15:56:36Z"
tags: [antichurn, integracao, bdd, cassettes, sqlite]
---

# Retomar

Integrei [#578](https://github.com/franklinbaldo/ies/pull/578) na `main`, merge [`86f2a4e`](https://github.com/franklinbaldo/ies/commit/86f2a4e27fb00d30c70485eef1b37fbc1a06da7f), após a correção mínima de quatro `needless_borrow` do Clippy. O head `32cc66a` passou [CI oficial 7/7](https://github.com/franklinbaldo/ies/actions/runs/42648911945) e CI rápido. A CLI real recusa título do conteúdo divergente da árvore via TLS local/anti-IES/SQLite, sem publicar corpo nem repetir mutação. **PASS sintético;** inspeção do card UnB de 27 interações e hashes OK no [Actions](https://github.com/franklinbaldo/ies/actions/runs/42647739691), mas a corrida de título não está capturada: **confronto semântico da transição NOT_RUN**; treinamento **NOT_RUN**.

Na última consulta, [#579](https://github.com/franklinbaldo/ies/pull/579) voltou a **Draft** com RED da Lista de Ciências ainda invisível; traz migração **v23 ainda não integrada**. [#577](https://github.com/franklinbaldo/ies/pull/577) também está Draft, com hipótese de conflito entre unidades; [#526](https://github.com/franklinbaldo/ies/pull/526) permanece Draft/conflitante. Se a #579 ocupar v23, a criação da #526 precisará da próxima versão (v24); **recalcular na `main` efetiva**, sem repetir comentários antes de haver fato novo. Nenhuma branch alheia alterada.

No [Cafezinho](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6744520943), a imagem da prensa ajuda a lembrar: título visto na árvore não garante o título visto no corpo depois. É analogia para uma hipótese testada localmente, não gravação independente do SEI.
