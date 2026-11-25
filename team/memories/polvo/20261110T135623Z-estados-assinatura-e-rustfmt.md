---
type: AgentMemory
title: "Assinatura documental: não confundir possibilidades com efeitos comprovados"
description: "PR #670 separa prévia, já assinado e assinatura comprovada por um enum com uma assinatura relida; correção de Rustfmt no CI; evidências externas não foram executadas."
agent: polvo
timestamp: "2026-11-10T13:56:23Z"
tags: [documento, assinatura, estados-tipados, rust, ci, anti-ies, cassettes]
---

# Para retomar

- **Fato:** na [issue #563](https://github.com/franklinbaldo/ies/issues/563), `DocumentoAssinado` admitia em seu tipo flags incompatíveis. Na [PR #670](https://github.com/franklinbaldo/ies/pull/670), substituí as flags por `ResultadoDaAssinatura` com prévia, assinatura anterior e `Comprovada` (carrega uma assinatura única relida e edição posterior). JSON/humano continuam projetando os nomes públicos anteriores. A derivação não altera POST, transporte, anti-IES ou SQLite.
- **Prova em curso:** o [CI rápido #42828635321](https://github.com/franklinbaldo/ies/actions/runs/42828635321) falsificou apenas três pontos de Rustfmt, que corrigi no head `7c942c2497dcd4d1c65d4044d2b53621c329d1ea`. O [CI oficial #42829068089](https://github.com/franklinbaldo/ies/actions/runs/42829068089) foi acionado nesse SHA; conferir jobs antes de alegar GREEN. BDD funcional novo/RED funcional **não executados** nesta refatoração.
- **Cassette-first:** card [UnB documento-escritas](https://github.com/franklinbaldo/ies/blob/main/cassettes/unb/documento-escritas-unb--36a904749e20.md) e evidência da assinatura lidos antes da alteração. **Confronto semântico bilateral NOT_RUN**; treinamento **NOT_RUN**. Os testes de fluxo sintético existentes foram adaptados, mas só contam como PASS após CI.
- **Cafezinho:** [reflexão no #388](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6769131174) sobre dois selos incompatíveis na mesma margem. Uma possibilidade admitida pelo tipo não é fato visto no SEI.
- **Continuidade:** verificar CI e parecer 🦅; sincronizar a branch com a `main` atual por merge de dois pais antes do handoff 🐘/🦡, sem force-push.
