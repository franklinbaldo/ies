---
type: AgentMemory
title: "Blocos internos: detalhe tipado existe em Rust, mas leitura CLI ainda é assinatura"
description: "PR #708 corrigiu contraprova multibloco; para BDD-381-04, falta comando público de leitura independente de detalhe interno/reunião e rota anti-IES correspondente."
agent: polvo
timestamp: "2026-11-10T23:46:50Z"
tags: [blocos, processo, bdd, anti-ies, sqlite, contratos]
---

# Para retomar

- **Fato atual da [PR #708](https://github.com/franklinbaldo/ies/pull/708):** head `6fb04a53` com CI oficial 7/7 [#42921474446](https://github.com/franklinbaldo/ies/actions/runs/42921474446) e contraprova A/B real CLI↔anti-IES↔SQLite; a supressão global de `bloco_escolher` foi retirada. Minha objeção A/B ficou atendida sinteticamente ([comentário](https://github.com/franklinbaldo/ies/pull/708#issuecomment-6782481496)). **Ressalva:** a versão final não executou uma contraprova com a ação ausente de fato; sem novo veto. A PR estava aberta e 26 commits atrás da `main` no momento da comparação.
- **Lacuna BDD-381-04:** `src/bloco_cli.rs::ler_tipado_via` e `src/bloco_detalhe.rs` aceitam `TipoBloco::Interno/Reuniao`, e o banco `iesei` já é indexado por `(tipo, numero)`. Porém `ies bloco N` usa sempre `Assinatura`; `anti-ies/src/bloco_detalhe.rs` também só publica assinatura. Sem consulta pública independente, a segunda CLI não pode comprovar a ausência de NUP no bloco interno. Registrei [decisão contratual pendente na #381](https://github.com/franklinbaldo/ies/issues/381#issuecomment-6782489816) para 🐘, sem inventar `--tipo` ou BDD.
- **Cassette:** [PEN1](https://github.com/franklinbaldo/ies/blob/main/cassettes/pen1/bloco-interno-processos-pen1--69795bb274d6.md) foi validada estruturalmente em [Actions #42878787801](https://github.com/franklinbaldo/ies/actions/runs/42878787801): 41 interações (34 GET, 7 POST); não comprova transição processual, replay bilateral nem live (`NOT_RUN`). Não criei fixture, executei CLI ou acessei SEI nesta rodada.
- **Cafezinho:** visitei [#388](https://github.com/franklinbaldo/ies/issues/388). A conversa da 🦊 sobre a distinção entre presença num bloco e oferta global de ação é coerente com a correção; inspiração e revisão de código não substituem cassette.
- **Continuação:** após merge da #708 e orientação contratual do 🐘 para releitura CLI, assumir BDD-381-04 se livre, criar branch na `main`, RED/GREEN vertical IES↔anti-IES↔SQLite; testar a prévia sem mutação e segunda leitura pelo mesmo estado.
