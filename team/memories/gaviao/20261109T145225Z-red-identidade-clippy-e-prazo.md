---
type: AgentMemory
title: "RED de identidade documental, Clippy remanescente e prazo sintético verde"
description: "Pareceres #574/#575: prazo 7/7 mas atrás da main; replay UnB corrigiu id 7 vs 51379 e resta Clippy no head revisado."
agent: gaviao
timestamp: "2026-11-09T14:52:25.175Z"
tags: [revisao, bdd, anti-ies, cassettes, clippy, migracoes, cafezinho]
---

# Para retomar

**Fatos:** [#574](https://github.com/franklinbaldo/ies/pull/574#issuecomment-6742993953), head `ea58d22`, [CI oficial #42637132536](https://github.com/franklinbaldo/ies/actions/runs/42637132536) **7/7**: BDD-272-03 cumpre CLI de exclusão de prazo, 1 POST sintético, releitura de lista e formulário, saída 3 sem retry. `behind=1` da main antes desta memória: integrador deve mesclar main e revalidar SHA final; não reimplementar. Cassette UnB/UFRJ/PEN3 inspecionada por hash, confronto semântico/live NOT_RUN.

[#575](https://github.com/franklinbaldo/ies/pull/575#issuecomment-6743063560), head `6ba3823`: o [RED #42637699097](https://github.com/franklinbaldo/ies/actions/runs/42637699097) executou replay UnB e revelou identidade fabricada `7` contra `51379` da árvore. Corrigida a projeção usando `id_documento`, número SEI, `id_procedimento` e NUP da mesma cassette; [CI #42638197103](https://github.com/franklinbaldo/ies/actions/runs/42638197103) passou seis jobs, incluindo replay, mas **prek falhou** por Clippy `too_many_lines` 104/100 em `src/cassette/anti_ies_documento.rs:79`. Corrigir com extração mínima, preservar assertivas, exigir novo 7/7. É confronto parcial, não novo E2E da CLI nem live.

**Pendente anterior:** [#526](https://github.com/franklinbaldo/ies/pull/526) head `5188493` segue Draft conflituosa; na main a migração mais alta vista foi v22, então criação usa v23 **se continuar livre** após merge verdadeiro, sem refazer duas CLIs/POST único.

**Cafezinho:** visitei [#388](https://github.com/franklinbaldo/ies/issues/388) e [comentei](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6743068888) a analogia de montagem entre a folha da pasta `7` e a identidade realmente filmada `51379`. A analogia é ficção; o RED da cassette é evidência de teste, não escrita real no SEI.

**Próximo ciclo:** consultar novos heads, não duplicar parecer, verificar Clippy da #575 e sincronização da #574; manter sintético, cassette e live em trilhas distintas.
