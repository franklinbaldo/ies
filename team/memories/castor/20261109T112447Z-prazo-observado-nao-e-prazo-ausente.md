---
type: AgentMemory
title: "Prazo observado não é o mesmo que prazo ausente"
description: "A CLI real passou em três estados de prazo no anti-IES; distinguir nenhuma observação de SemPrazo evita falsos negativos no simulador."
agent: castor
timestamp: "2026-11-09T11:24:47Z"
tags: [prazo, bdd, cli, anti-ies, sqlite, observabilidade]
---

# Para retomar

**Fato:** na [PR #560](https://github.com/franklinbaldo/ies/pull/560), a [issue #272](https://github.com/franklinbaldo/ies/issues/272) ganhou esquema SQLite efêmero v20 e projeção GET tipada. O teste usa **CLI real → TLS local → anti-IES → SQLite** para `SemPrazo`, `Data` e `Concluido`, sem POST. [RED #42607289676](https://github.com/franklinbaldo/ies/actions/runs/42607289676) registrou uma falha; [GREEN #42608259475](https://github.com/franklinbaldo/ies/actions/runs/42608259475) passou 7/7.

**Aprendizado:** `None` (sem testemunha do prazo) não se confunde com `Some(SemPrazo)` (testemunha da ausência de prazo). Essa distinção ganhou uma analogia no [Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6738852533). Inspiração lógica não é prova de SEI: hashes UnB/GO verificados, confronto semântico e treinamento `NOT_RUN`.

**Retomar:** primeiro revisão/integração #560, com merge verdadeiro da main se necessário; depois BDD-272-02/03. Escrita/exclusão continuam fechadas antes da rede. Não refazer o BDD-272-01 nem introduzir subcódigos.
