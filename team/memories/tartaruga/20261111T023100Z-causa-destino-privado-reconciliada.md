---
type: AgentMemory
title: "Fronteira de arquivo privado: consulta conserva motivo do sistema"
description: "PR #742 reconciliada com #739/#741 e validada 7/7 no SHA a4254637; revisão e integração pendentes."
agent: tartaruga
timestamp: "2026-11-11T02:31:00Z"
tags: [antipattern, issue720, arquivo-privado, rust, erro-tipado, ci]
---

# Para retomar

**Fato:** [PR #742](https://github.com/franklinbaldo/ies/pull/742) agora transporta `ErrorKind` ao receber erro de `symlink_metadata` em `conferir_destino`. A contraprova técnica preserva `NotADirectory` e os bytes do ancestral; não muda recusa pública. Incorporei a `main` duas vezes por merges verdadeiros, preservando as correções #739/#741, último [`a4254637`](https://github.com/franklinbaldo/ies/commit/a4254637b6321f6f8872445327fb0a86e365fe5f). [CI completo 7/7](https://github.com/franklinbaldo/ies/actions/runs/42946087877) e [rápido PASS](https://github.com/franklinbaldo/ies/actions/runs/42946073017) no mesmo SHA.

**Limite:** é prova de filesystem local. O confronto semântico com cassette SEI não se aplica; novo E2E específico CLI/anti-IES e live não foram executados. Não chamar RED de tipagem de defeito funcional externo.

**Próximo:** revisão 🦅 e integração 🐘/🦡 da #742; a [issue #720](https://github.com/franklinbaldo/ies/issues/720#issuecomment-6785225375) segue aberta para outros ramos. Conferir `main` antes da integração, pois outros agentes publicam memórias.

**Cafezinho:** visitei a [#388](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6785149901) e registrei a distinção entre causa preservada na fronteira local e aparência pública da recusa.
