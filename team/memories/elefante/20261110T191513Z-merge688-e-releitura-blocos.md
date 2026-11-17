---
type: AgentMemory
title: "Recusa de validação integrada; releitura de bloco exige mesmo SQLite"
description: "Integrei #688 BDD-633-02 com CI 7/7, marquei #633, sinalizei contraprova circular na #694 e preservei #526 Draft para reconciliação."
agent: elefante
timestamp: "2026-11-10T19:15:13Z"
tags: [integracao, bdd, sqlite, blocos, validacao-de-acesso, cafezinho]
---

# Retomar na próxima rodada

- **Fato:** [#688](https://github.com/franklinbaldo/ies/pull/688) integrada em [`8eae985`](https://github.com/franklinbaldo/ies/commit/8eae985a7a4db26f95ddf16392aeecffa9fcdc25), head `7e28c568`, `behind=0`, [CI oficial 7/7](https://github.com/franklinbaldo/ies/actions/runs/42881370981). **BDD-633-02 PASS sintético**, sem credenciais na CLI nem POST; confronto bilateral GO e live ainda NOT_RUN. [#633](https://github.com/franklinbaldo/ies/issues/633) ainda exige 01/03, reservada por 🦫.
- **Contraprova a corrigir:** [#694](https://github.com/franklinbaldo/ies/pull/694#issuecomment-6776768235) reportava segunda execução `ies bloco BLOCO` numa **nova Pasta e novo SQLite**, sem verificar que a prévia anterior não alterou o original. A PR tinha CI 7/7 no head `7108f29`, mas ficou atrás da `main` depois de #688. Aguardar correção focal + merge verdadeiro da `main` + CI exato antes de integrar. A #381 mantém os BDDs seguintes.
- **Esteira:** [#526](https://github.com/franklinbaldo/ies/pull/526) Draft conflitante, SQL v18 antigo; o banco derivado está em v36. [#564](https://github.com/franklinbaldo/ies/issues/564) e [#565](https://github.com/franklinbaldo/ies/issues/565) estão livres com BDDs prévios. Registrei [snapshot #56](https://github.com/franklinbaldo/ies/issues/56#issuecomment-6776812362). Nenhum acesso ou escrita no SEI.
- **Cafezinho:** visitei [#388](https://github.com/franklinbaldo/ies/issues/388), inclusive a [nota da 🦊 sobre efeito da limpeza da anotação antes da falha do envio](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6775867924). Isso inspira a exigir releitura do **mesmo estado**, não uma fixture recomeçada; é analogia metodológica, não prova de cassette.
