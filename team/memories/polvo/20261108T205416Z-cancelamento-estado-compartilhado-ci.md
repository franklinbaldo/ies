---
type: AgentMemory
title: "BDD-273-02: cancelamento da CLI persiste nas rotas do anti-IES"
description: "PR #509 pronta e CI 7/7 no head 92f1715; regressão prova SQLite, árvore e recusa de segunda escrita após a CLI real."
agent: polvo
timestamp: "2026-11-08T20:54:16Z"
tags: [documento, cancelamento, anti-ies, sqlite, bdd, ci, regressao]
---

# Retomar

- A [PR #487](https://github.com/franklinbaldo/ies/pull/487) foi integrada à `main` em 08/11/2026, às 19:58 UTC. Não voltar a sincronizar a migração v13 como se ainda estivesse pendente.
- Nova [PR #509](https://github.com/franklinbaldo/ies/pull/509), **Ready** no head `92f1715448fe1b1e53c3c8b45ee78ad59ced0168`, inclui teste técnico do BDD-273-02: a CLI real cancela no anti-IES (10 requisições sintéticas), o SQLite independente retém o motivo, a árvore mostra cancelamento e omite a ação, GET/POST posterior de cancelamento é recusado sem alterar estado. Não é BDD novo nem implementação duplicada.
- [CI completo #42522146158](https://github.com/franklinbaldo/ies/actions/runs/42522146158) **7/7 SUCCESS** no head exato; [CI rápido #42522119288](https://github.com/franklinbaldo/ies/actions/runs/42522119288) verde. Corrigido Rustfmt após primeira falha de CI rápido. RED comportamental não foi executado, pois a regressão comprova o comportamento existente. [Handoff na #273](https://github.com/franklinbaldo/ies/issues/273#issuecomment-6726984606).
- Cassette-first: lidos cards GO e UFRJ `documento-mover-cancelar` (três operações de cancelamento por card). Nesta rodada, ZIP/replay semântico **NOT_RUN**; live **NOT_RUN**; somente a integração sintética passou. Sem REA, credenciais, novos subcódigos ou escrita SEI.
- No [Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388), a metáfora recente do 🦡 (fuzz→compressor vs compressor→fuzz) reforçou a distinção entre vestígio final e cronologia identificável. Inspiração ficcional, não evidência SEI.
- **Próxima ação:** 🦅 revisão e 🐘/🦡 integração da #509, sem reimplementar a #487; seguir para confronto semântico independente com cassettes ou BDD documental ainda pendente.
