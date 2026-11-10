---
type: AgentMemory
title: "Revisões de busca opaca, envio recusado, inclusão em bloco e modal"
description: "Quatro pareceres: #691 verde 7/7; #693 funcional mas CI do novo head pendente; #694 tem segunda CLI em outro SQLite; #688 verde e atrás da main."
agent: gaviao
timestamp: "2026-11-10T18:46:42Z"
tags: [revisao, bdd, documento, bloco, envio, modal, sqlite, ci, cassettes, cafezinho]
---

# Retomar

- **#691:** [parecer](https://github.com/franklinbaldo/ies/pull/691#issuecomment-6776134986) no head `639a4b5`; [CI 7/7](https://github.com/franklinbaldo/ies/actions/runs/42877106892), `behind=0` antes desta memória. A pesquisa sintética aceita omissão de zeros, mas a CLI real exige identidade publicada literal; live UnB de 04/11 é anterior, não novo ensaio.
- **#693:** [parecer](https://github.com/franklinbaldo/ies/pull/693#issuecomment-6776135632) no head `2477488`, CI rápido verde e oficial pendente; BDD-581-02 preserva `anotacao_depois:false` após recusa de envio. Detectei remoção acidental do contrato `bloco-incluir` no head anterior, **corrigida** antes do parecer. O head avançou posteriormente para `878b387` só com nova memória da 🦊, exigindo novo CI.
- **#694:** [parecer](https://github.com/franklinbaldo/ies/pull/694#issuecomment-6776136345) no head `0b9cf83`, CI 7/7: em `tests/aies_documento_cli/testes_bloco_incluir.rs`, a segunda consulta roda em `Pasta::nova()` e re-semeia **outro** SQLite. Ajustar teste para duas CLIs sobre o **mesmo banco**, como exige BDD-381-01. Não é veto por falta de cassette.
- **#688:** [parecer](https://github.com/franklinbaldo/ies/pull/688#issuecomment-6776137088) no head `9b3143d`, CI 7/7, `behind=5`; mensagem para `IES_CREDENCIAIS_FILE` ausente corrigida, BDD-633-02 sintético verde.

**Evidência:** preservar separação entre teste sintético, replay bilateral de cassette e live; hashes não são paridade. Não houve acesso ao SEI, credenciais, produção, implementação ou merge. Visitei o [Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388), incluindo a [nota 🦊 da #693](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6775867924) sobre evento comprovado e envio recusado. Próxima rodada: conferir novos heads, 7/7 e reconciliação antes da integração.
