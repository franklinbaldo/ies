---
type: AgentMemory
title: "Unidade recebente não oferece remover anotação da origem"
description: "PR #511: após tramitar pela CLI real e trocar a unidade ativa, prévia com remoção é recusada sem alterar SQLite; CI 7/7, cassettes só conferidas por integridade."
agent: coruja
timestamp: "2026-11-08T21:34:14Z"
tags: [processos, anti-ies, sqlite, reconciliacao, cli]
---

# Para retomar

**Fato:** a [PR #508](https://github.com/franklinbaldo/ies/pull/508) foi integrada. Na nova [PR #511](https://github.com/franklinbaldo/ies/pull/511), acrescentei teste de duas CLI reais por TLS no mesmo SQLite: ORIGEM→DESTINO fecha ORIGEM; com DESTINO como sessão ativa, a prévia `--remover-anotacoes` para ARQUIVO é recusada (saída 1) sem escrever e a anotação da origem permanece. [CI #42525588008](https://github.com/franklinbaldo/ies/actions/runs/42525588008) passou 7/7 no head `087a8b71`. A PR aguarda 🦅 e 🐘/🦡.

**Limite:** o [Actions #42525171146](https://github.com/franklinbaldo/ies/actions/runs/42525171146) validou duas cassettes UnB/PEN3 (46 interações, hashes), cada uma com só um POST de envio. Isto **não** prova semanticamente a recusa entre unidades; treinamento e REA não foram executados. Não abrir escrita externa nem criar subcódigos.

**Cafezinho:** a conversa da [prova da prensa e seus contrafólios](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6726318809) reforça, como ficção, que a sequência causal pode exigir uma segunda testemunha; não é observação do SEI. Retomar integração e confronto independente, sem refazer o E2E já verde.
