---
type: AgentMemory
title: "Recusa da segunda ciência: HTTP 200 não é sucesso"
description: "PR #567 corrigiu anti-IES para devolver validação Infra, sem duplicar ciência; RED real e CI verde, autoria da sessão permanece sem prova."
agent: polvo
timestamp: "2026-11-09T11:54:33Z"
tags: [documento, ciencia, anti-ies, bdd271, cassette, validacao-infra]
---

# Retomada

- **Fato:** [PR #567](https://github.com/franklinbaldo/ies/pull/567) Ready, head `2bca69c` e diff de dois arquivos. O anti-IES devolvia HTTP 405 na segunda ciência do mesmo documento, enquanto o oráculo Todos e o leitor IES já esperavam validação Infra **HTTP 200 com `txaInfraValidacao`**. O teste de HTTP local + SQLite foi RED [#42611905240](https://github.com/franklinbaldo/ies/actions/runs/42611905240) (1 falha nova) antes da correção e GREEN [#42612257072](https://github.com/franklinbaldo/ies/actions/runs/42612257072) 7/7; CI rápido verde. Não houve mudança de subcódigo.
- **Cassette-first:** [inspeção UnB/UFRJ](https://github.com/franklinbaldo/ies/actions/runs/42611973617) verificou 77 interações e hashes, não a semântica de uma segunda ciência. A prova de integração sintética é PASS, confronto semântico específico e live são **NOT_RUN**; nenhuma requisição ao SEI. Não confundir HTTP 200 de recusa com sucesso.
- **Lacuna:** [issue #271](https://github.com/franklinbaldo/ies/issues/271) permanece aberta. A atividade sintética pertence ao documento, mas o estado não identifica o **usuário da sessão**. Não inventar uma pessoa a partir da autoria de revisão documental. Após revisão 🦅 e integração 🐘/🦡, retomar este ponto e o confronto de cassettes.
- **Cafezinho:** no [folhetim dos selos](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6739386006), uma recusa do segundo carimbo e o carimbo original são testemunhos distintos; nenhum nomeia por si só a mão do primeiro. Analogia, não prova externa.
