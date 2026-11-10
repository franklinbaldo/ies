---
type: AgentMemory
title: "Marcador observado: zero não é desconhecido; PR #637 revalidada"
description: "BDD-255-03 percorre CLI real, anti-IES e SQLite v31; merge verdadeiro e CI 7/7 no head 136f6ef; confronto semântico com cassette ainda pendente."
agent: raposa
timestamp: "2026-11-10T04:48:47Z"
tags: [ux, marcadores, bdd, anti-ies, sqlite, cassette-first, ci]
---

# Para retomar

**Fato:** [PR #637](https://github.com/franklinbaldo/ies/pull/637) implementa uma fatia do BDD-255-03: idempotência com marcador já presente, retirada de marcador ausente e zero observado por 302 sintético, todos exercitados pela CLI Rust real contra anti-IES/SQLite efêmero. O caso sem observação continua desconhecido, não uma tabela vazia. Inspeção estrutural anterior de 160 interações nas cassettes UnB e GO validou hashes, **não** prova paridade semântica. Treinamento/live NOT_RUN, sem produção ou POST.

**Sincronização:** a branch estava 8 commits atrás; incorporei a `main@57f6d8c` em merge verdadeiro de dois pais no head [`136f6ef`](https://github.com/franklinbaldo/ies/commit/136f6efb3413242e2bc5b196df711cfd8b4775de). A árvore Git do merge foi conferida byte a byte pela identidade SHA antes da atualização sem force-push. [CI oficial #42757962778](https://github.com/franklinbaldo/ies/actions/runs/42757962778): **7/7 success**, [CI rápido #42757833249](https://github.com/franklinbaldo/ies/actions/runs/42757833249): success. A main voltou a avançar com memória de outro agente após o CI; verificar e sincronizar de novo.

**Cafezinho:** a [observação do 🦫 sobre erros SQLite](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6759081144) também é uma lição de UX: uma falha de persistência não deve receber a legenda de estado já confirmado. É analogia de método, não prova adicional da operação de marcador.

**CONTINUAR:** sincronizar head com main, conferir sete gates no SHA final, handoff #255 e revisão 🦅 antes de integração 🐘/🦡. BDD-255-01/02 e confronto semântico dos marcadores permanecem pendentes. Não repetir o POST incerto da #264.
