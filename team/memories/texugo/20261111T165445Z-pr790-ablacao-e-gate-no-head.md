---
type: AgentMemory
title: "PR #790: teste vazio substituído; replay do head reconciliado em validação"
description: "Confirmei replay original completo no head anterior, reconciliei PR com main via merge verdadeiro e reativei CI oficial; a negativa de login é apenas sintética."
agent: texugo
timestamp: "2026-11-11T16:54:45.390Z"
tags: [antichurn, ablacao, cassette-e2e, github-actions, revisao]
---

# Retomar

**Fatos:** Li integralmente AGENTS.md e team/texugo.md na main, as cinco memórias recentes (nenhuma fixada) e consultei o Cafezinho #388. Na [PR #790](https://github.com/franklinbaldo/ies/pull/790), o [CI oficial #43052279406](https://github.com/franklinbaldo/ies/actions/runs/43052279406) concluiu **7/7 PASS** no SHA b4945875: log registra `RUSTUP_TOOLCHAIN=1.99.1 scripts/replay acervo`, 495 cards/29 ZIPs conferidos por hash, 34 replays acervo e 18 testes `cassette_` PASS. Este replay geral não prova a negativa de login, que é HTML sintético local.

**Fatos:** A main avançou oito commits. Reconciliei a PR por merge verdadeiro [e3574292](https://github.com/franklinbaldo/ies/commit/e357429245713a3d2cad00d7a06967a2e5a33617), pais b4945875 e main@91f8593b, sem force-push. Delta único `src/status_web.rs` (+11/-1), substituição de teste vazio. CI rápido no head e3574292 PASS; [CI oficial #43062726635](https://github.com/franklinbaldo/ies/actions/runs/43062726635) iniciado, ainda não certificado. Revisão 🦅 favorável tecnicamente. Registrei no acervo que o caso de login adversarial não tem card/ZIP identificado, logo as comparações específicas IES↔cassette / anti-IES↔cassette / CLI↔anti-IES↔SQLite são **NOT_RUN**. Treinamento NOT_RUN. Nenhuma escrita externa, novo gate ou BDD.

**Próximo ciclo:** como esta memória publicada na main causa nova divergência, sincronizar novamente apenas uma vez após a memória, antes de disparar o CI oficial definitivo; conferir replay executado e jobs no SHA final antes do squash. Não atribuir PASS sintético a paridade externa, nem exigir número fixo de GETs/POSTs.
