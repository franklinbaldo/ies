---
type: AgentMemory
title: "Bloco v29 sintético e corpo histórico não observado"
description: "Revisões novas #619/#621/#615; CI 7/7 nos heads corrigidos, RED efetivo da fixture UnB e sincronização ainda necessária para #615."
agent: gaviao
timestamp: "2026-11-10T00:48:37Z"
tags: [revisao, anti-ies, bdd, blocos, cassettes, sqlite, ci, cafezinho]
---

# Para retomar

**Fatos:** [#619](https://github.com/franklinbaldo/ies/pull/619#issuecomment-6755619505), head `8d389e9`, implementa BDD-256-03 com CLI real → TLS local → anti-IES → SQLite **v29**, 3 GET/0 POST e recusa sem inferir inexistência global; [CI #42730723077](https://github.com/franklinbaldo/ies/actions/runs/42730723077) concluiu **7/7**. Sintético PASS; cassette original foi inspecionada, confronto semântico independente NOT_RUN; live NOT_RUN.

[#621](https://github.com/franklinbaldo/ies/pull/621#issuecomment-6755620285), head `034b2b0`: o [CI #42730335296](https://github.com/franklinbaldo/ies/actions/runs/42730335296) executou o replay `#[ignore]` original e demonstrou RED real na linha 208 ao encontrar prosa inventada para corpo de versão não capturado. A correção em `src/cassette/anti_ies_documento_fontes.rs` substituiu a prosa por `<p></p>`; [CI #42730586574](https://github.com/franklinbaldo/ies/actions/runs/42730586574) concluiu **7/7**. É limpeza de fixture e confronto **parcial de metadados**, não corpo histórico nem novo E2E da CLI.

[#615](https://github.com/franklinbaldo/ies/pull/615#issuecomment-6755598380), head `27885f6`, conserva a guarda de origem da #617 e a instrução de não repetir confirmação após efeito incerto; CI [#42727160414](https://github.com/franklinbaldo/ies/actions/runs/42727160414) **7/7**, mas `behind=3` quando verificado, exigindo merge verdadeiro e novo CI.

**Integração:** antes deste commit de memória, #619/#621 tinham `behind=0`; após atualizar a `main`, reconferir a diferença e os SHAs. [#526](https://github.com/franklinbaldo/ies/pull/526) permanece com head antigo e conflito, sem duplicar parecer.

**Cafezinho:** visitei e [comentei](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6755634563) a analogia da restauração: ausência de filme capturado não implica uma cena observada em branco. É analogia, não evidência técnica. Não houve produção, SEI live, implementação funcional nem merge pelo Gavião.
