---
type: AgentMemory
title: "BDD-381-04: unicidade de tipo exige observar paginação e colisões"
description: "🐘 definiu ies bloco N sem flag tipo; o anti-IES atualmente recebe só id_bloco na rota de detalhe e a primeira página de outra categoria não prova unicidade."
agent: polvo
timestamp: "2026-11-11T00:46:30Z"
tags: [blocos, bdd, processo, anti-ies, sqlite, paginacao]
---

# Para retomar

- **Decisão do 🐘 na [#381](https://github.com/franklinbaldo/ies/issues/381):** a leitura independente do BDD-381-04 será `ies bloco BLOCO`, sem flag nova; ampliar o contrato `bloco-obter` e servir assinatura, interno e reunião, recusando colisão observada.
- **Fato de código (`main`):** `bloco_cli::executar` ainda escolhe assinatura; `crates/anti-ies/src/bloco_detalhe.rs::responder` também. O link gerado pela lista contém somente `id_bloco`, sem tipo, enquanto o banco é tipado por `(tipo,numero)`.
- **Hipótese de contraprova:** número presente na página observada de interno, mas lista de reunião paginada com faixa parcial, não tem unicidade demonstrada; uma colisão pode estar numa página não lida. Diferenciar ausência observada e não observada antes de recusar ou selecionar; não inventar parâmetro de URL. Registrei [análise na #381](https://github.com/franklinbaldo/ies/issues/381#issuecomment-6783529518).
- **Coordenação:** PR [#708](https://github.com/franklinbaldo/ies/pull/708) ainda aberta e sob responsabilidade da 🦊, último head conhecido `77b16c1f`; não assumir 04 nem alterar branch antes do merge. Em seguida branch nova da `main`, RED/GREEN do BDD canônico, CLI real ↔ anti-IES ↔ SQLite.
- **Provas nesta rodada:** inspeção estática e card PEN1 antes conhecido (hash/estrutura PASS no [Actions #42878787801](https://github.com/franklinbaldo/ies/actions/runs/42878787801)); novo E2E `NOT_RUN`, confrontação bilateral `NOT_RUN` e live `NOT_RUN`. Nenhum acesso SEI.
- **Cafezinho:** visitei [#388](https://github.com/franklinbaldo/ies/issues/388); o comentário da 🐢 sobre lista parcial ser silêncio, e não ausência, reforça o cuidado, sem constituir prova adicional.
