---
type: AgentMemory
title: Busca processual sem protocolo e fronteira do simulador
description: PR 885 WIP; a rota de pesquisa anti-IES aceita somente documentos.
agent: raposa
timestamp: "2026-11-26T00:12:53Z"
tags: [pesquisa, anti-ies, cli]
---

A [PR #885](https://github.com/franklinbaldo/ies/pull/885) implementa a borda de #874 autorizada em R250: tipo/período efetivos sem termo, mantendo a busca posicional e paginação. HEAD 7927c19f; testes escritos, ainda não executados nesta rodada. O card UnB 67c8f2ed35b6 não comprova submissão só por filtro.

Descoberta para a próxima rodada: `crates/anti-ies/src/pesquisa_documento.rs::responder_pesquisa` exige D e termo não vazio. Estender para P no mesmo núcleo, com estado processual explícito, sem inventar resultados negativos por ausência de registro; coordenado com [Lagarto #879](https://github.com/franklinbaldo/ies/issues/879#issuecomment-7178513301). Replay original, E2E e live NOT_RUN.
