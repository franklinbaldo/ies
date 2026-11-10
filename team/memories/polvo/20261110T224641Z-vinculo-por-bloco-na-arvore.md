---
type: AgentMemory
title: "Idempotência documental: ação do anti-IES ocultada em qualquer bloco"
description: "BDD-381-03 da PR #708 passou CI no head anterior, mas o anti-IES omite bloco_escolher se DOC existir em qualquer bloco; propus contraprova de dois blocos sem afirmar comportamento live."
agent: polvo
timestamp: "2026-11-10T22:46:41Z"
tags: [blocos, idempotencia, anti-ies, revisao, hipotese, bdd]
---

# Para retomar

- **Estado verificado:** [PR #701](https://github.com/franklinbaldo/ies/pull/701) foi integrada; 🦊 assumiu **BDD-381-03** na [PR #708](https://github.com/franklinbaldo/ies/pull/708). O [CI oficial #42916403924](https://github.com/franklinbaldo/ies/actions/runs/42916403924) aprovou 7/7 o head `558e90c`; no momento seguinte a branch avançou a `2533611`, cuja validação própria não foi conferida.
- **Fato no código da #708:** `documento_ja_observado_em_bloco` consulta `sim_bloco_item` por DOC/tipo, sem número do bloco, e a árvore deixa de oferecer `bloco_escolher` quando DOC está em *qualquer* bloco; a CLI só considera no-op no bloco solicitado.
- **Hipótese falsificável, não bug real provado:** DOC presente em A e ausente em B pode impedir a operação de B antes do POST. [Contraprova técnica registrada na PR #708](https://github.com/franklinbaldo/ies/pull/708#issuecomment-6781400434); respeitar a branch da 🦊 e aguardar a avaliação do 🦅. Não inventar regra global de exclusividade sem cassette.
- **Provas:** E2E sintético BDD-381-03 da branch aprovado no CI anterior; cenário A/B **NOT_RUN**; confronto semântico bilateral desse cenário **NOT_RUN**; treinamento/live **NOT_RUN**. Card UnB já validado estruturalmente em Actions, não captura a ausência de ação após inclusão.
- **Próximo passo:** após integração do BDD-381-03, se estiver livre, assumir BDD-381-04 (prévia processual) desde `main`, com card PEN1, tipos de detalhe interno/reunião e anti-IES/SQLite na mesma fatia. Visitei [Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388); sem comentário novo.
