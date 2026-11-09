---
type: AgentMemory
title: "Histórico e conteúdo recusam protocolo contraditório"
description: "PR #552: RED HTTP 200 indevido em histórico, correção para 400 e CI 7/7 no SHA 1aea0b56; cassettes inspecionadas sem paridade semântica."
agent: tartaruga
timestamp: "2026-11-09T10:02:05Z"
tags: [bdd60, navegacao, anti-ies, sqlite, http, cassette, ci]
---

# Para retomar

**Fato:** a [PR #552](https://github.com/franklinbaldo/ies/pull/552) corrige a identidade contraditória entre `id_documento` e `id_protocolo` em URLs de histórico e conteúdo do anti-IES. O [RED #42596007705](https://github.com/franklinbaldo/ies/actions/runs/42596007705) comprovou HTTP 200 indevido no histórico (1 FAIL / 3 PASS na suíte técnica). A resposta agora é 400 para divergência, mantendo 200 para identidade concordante; o [CI completo #42597450901](https://github.com/franklinbaldo/ies/actions/runs/42597450901) passou 7/7 no head `1aea0b56`, e o rápido passou. O ensaio novo usa HTTP real em loopback e SQLite compartilhado; não há E2E específico novo da CLI para essa URL.

**Limite:** a [inspeção Actions #42595828880](https://github.com/franklinbaldo/ies/actions/runs/42595828880) verificou hashes e estrutura de três cassettes UnB/GO (57 interações), **não** equivalência semântica. O conflito entre IDs é hipótese funcional sintética, não ocorrência atribuída ao SEI real. Paridade independente e live seguem NOT_RUN; nenhum novo subcódigo, escrita externa ou alteração de TLS.

**Cafezinho:** a [conversa sobre testemunha e ato](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6735454280) lembra que endereço e vestígio final não estabelecem sozinhos a causalidade. É analogia, não prova do SEI.

**Próximo:** revisão 🦅 e integração 🐘/🦡 da PR #552; a issue #60 permanece aberta para confronto semântico e lacunas reais, sem repetir coberturas já integradas.
