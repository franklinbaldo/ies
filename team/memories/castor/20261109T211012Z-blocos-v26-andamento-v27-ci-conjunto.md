---
type: AgentMemory
title: "Blocos v26 e andamento v27 encaixados com CI conjunto"
description: "PR #602 reconciliou a #597 por merge verdadeiro, reservou v27 ao andamento e passou CI 7/7; resta revisão/integração e o confronto semântico."
agent: castor
timestamp: "2026-11-09T21:10:12Z"
tags: [sqlite, anti-ies, andamento, bdd, ci, merge, cassettes]
---

# Próxima rodada

**Fato:** [PR #602](https://github.com/franklinbaldo/ies/pull/602), head `e6a9c739`, absorveu por merge de dois pais (`83698c4`) a [#597](https://github.com/franklinbaldo/ies/pull/597) já integrada. Blocos conserva `0026_blocos_listados.sql`; andamento passou para `0027_andamento_processo.sql`, com `user_version=27` e expectativas atualizadas. O primeiro CI rápido encontrou apenas a ordem de módulos Rustfmt, corrigida. [CI oficial #42699121943](https://github.com/franklinbaldo/ies/actions/runs/42699121943): **7/7 PASS**; rápido e inspeção de duas cassettes, 50 interações, também PASS.

**Limite:** BDD-259-01 tem execução sintética da CLI real ↔ anti-IES ↔ SQLite, sem POST. Inspeção das cassettes comprova hashes/estrutura, **não** paridade semântica; confronto independente e treinamento seguem `NOT_RUN`. `--confirmar` público continua bloqueado. [Handoff #259](https://github.com/franklinbaldo/ies/issues/259#issuecomment-6751536103).

**Retomar:** revisão 🦅 e integração 🐘/🦡 da #602, novamente sincronizando a `main` se avançar e exigindo CI no SHA final. Só depois avançar BDD-259-02/03.

**Cafezinho:** a distinção do [🦅 entre objeto encontrado e legenda equivocada](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6751015072) lembra que dois programas concordarem não demonstra que os dois reproduzem o observado. É inspiração, não prova de SEI.
