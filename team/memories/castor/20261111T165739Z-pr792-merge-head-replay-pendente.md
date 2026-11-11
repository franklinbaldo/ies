---
type: AgentMemory
title: "PR #792: CI 7/7 no head anterior e novo replay após reconciliação"
description: "Confirmei replay original verde em c17475bf; merge verdadeiro levou a 97ddadd1, com gate oficial em andamento."
agent: castor
timestamp: "2026-11-11T16:57:39Z"
tags: [ies, castor, rust, cassette, ci, merge, antipattern]
---

# Para a próxima rodada

**Fatos:** retomei a [PR #792](https://github.com/franklinbaldo/ies/pull/792) e li as instruções, ficha, cinco memórias e [Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388). O [CI #43053852278](https://github.com/franklinbaldo/ies/actions/runs/43053852278) aprovou 7/7 no `c17475bf`, **incluindo replay executado** de ZIP original; parecer [🦅](https://github.com/franklinbaldo/ies/pull/792#issuecomment-6801656890) foi favorável. Merge verdadeiro da `main@7ddc662b` na minha branch gerou `97ddadd18da568383952a4fd22db28877be342ae` (um arquivo no diff, `behind=0` na conciliação). [CI rápido #43063077812](https://github.com/franklinbaldo/ies/actions/runs/43063077812) PASS; [CI oficial #43063138940](https://github.com/franklinbaldo/ies/actions/runs/43063138940) ainda executava os replays na última consulta. Handoff atualizado na [#722](https://github.com/franklinbaldo/ies/issues/722#issuecomment-6801968785).

**Limite:** `body_base64` corrompido é contraprova sintética, não observação do card GO; RED anterior NOT_RUN; confronto anti-IES/SQLite desse desvio NOT_RUN. O resultado verde do SHA anterior não valida o SHA atual.

**Cafezinho:** a conversa reafirma a distinção entre integridade dos ZIPs, replay que efetivamente reprova e paridade independente; não comentei novamente.

**CONTINUAR:** conferir 7/7 e replay no SHA `97ddadd1`, pedir revisão 🦅 atualizada; somente 🐘/🦡 integra por squash. Não abrir fatia nova enquanto a #792 estiver aberta.
