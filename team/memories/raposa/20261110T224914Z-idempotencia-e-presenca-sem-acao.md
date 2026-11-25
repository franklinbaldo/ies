---
type: AgentMemory
title: "BDD-381-03: presença comprovada independe da oferta de inclusão"
description: "PR #708 no merge verdadeiro 25336111, RED reproduzido e CI final 7/7; rever com Gavião e integrar antes do BDD-381-04. Confronto bilateral e live ainda não realizados."
agent: raposa
timestamp: "2026-11-10T22:49:14Z"
tags: [ux, bloco, bdd-381-03, idempotencia, anti-ies, sqlite, cassette-first, ci]
---

# Para retomar

Fato: a [PR #708](https://github.com/franklinbaldo/ies/pull/708) está Ready, mergeável, no SHA `25336111`, reconciliada por merge verdadeiro com a `main`. O [RED #42913537769](https://github.com/franklinbaldo/ies/actions/runs/42913537769) reproduziu a CLI exigindo ação antes de ler pertencimento. O [CI final #42917439869](https://github.com/franklinbaldo/ies/actions/runs/42917439869) passou 7/7 e o [rápido #42917294216](https://github.com/franklinbaldo/ies/actions/runs/42917294216) passou. CLI real → anti-IES → mesmo SQLite comprova idempotência, inclusive sem `bloco_escolher`, sem POST e sem duplicação; recusa de bloco interno também foi testada. Esperar revisão independente 🦅 e integração 🐘/🦡 antes do BDD-381-04.

Limite: a [cassette UnB](https://github.com/franklinbaldo/ies/blob/main/cassettes/unb/bloco-incluir-anotar-retirar-unb--c3232a87e814.md) tem 152 interações e hash conferido no [Actions #42889943882](https://github.com/franklinbaldo/ies/actions/runs/42889943882), mas não registra a ausência da ação após pertencimento: confronto semântico bilateral **NOT_RUN**, treinamento/live **NOT_RUN**. Hipótese de bancada não vira fato do SEI. A `main` avançou após o merge da branch somente com memória OKF alheia.

Na [conversa do Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6781228758), a imagem da estante sem botão de adicionar resume o princípio: oferta de verbo é interface, presença observada é estado. Não tratá-los como sinônimos.
