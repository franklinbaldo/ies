---
type: AgentMemory
title: "Confirmar adição não autoriza perder vínculos anteriores"
description: "RED flagrou confirmação falsa com perda colateral; GREEN passa a exigir destino e relações anteriores na releitura, sem liberar escrita da CLI."
agent: castor
timestamp: "2026-11-09T01:14:59Z"
tags: [rust, relacionamentos, bdd, anti-ies, causalidade, red-green]
---

# Próximo encaixe

**Fato:** na [PR #524](https://github.com/franklinbaldo/ies/pull/524), a adição de vínculo dizia `Confirmado` ao encontrar apenas o novo destino, mesmo se outro vínculo sumisse. O [RED #42543301059](https://github.com/franklinbaldo/ies/actions/runs/42543301059) teve **1 falha / 2.125 aprovações**. O [GREEN final #42544124763](https://github.com/franklinbaldo/ies/actions/runs/42544124763) passou **7/7** após merge verdadeiro da main: conferir novo destino e todos os anteriores por id+NUP; só um POST mutador e uma GET posterior. Sem criar erro ou esquema paralelo.

**Limite da prova:** testes técnicos Rust sintéticos PASS; [inspeção PEN1 #42543641770](https://github.com/franklinbaldo/ies/actions/runs/42543641770) validou hashes e 25 interações, não confronto semântico. BDD-258-02 por CLI real ↔ anti-IES ↔ SQLite, confronto com cassette e live continuam NOT_RUN. `--confirmar` permanece bloqueado pré-rede.

**Retomar:** revisão 🦅 e integração 🐘/🦡 da #524; depois BDD-258-02/03/04, sem reabrir BDD-258-01. No [Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6729399043), a 🦊 distinguiu sequência causal e sua projeção final: analogia útil para a releitura independente, não evidência material da prensa nem do SEI.
