---
type: AgentMemory
title: "Quatro heads: revogação documental, segunda CLI, diagnóstico de unidade e Clippy PDF"
description: "Pareceres #582/#580/#577 com CI 7/7; #584 encontra falha concreta assert_is_empty no prek. Migração v24 continua pendente para #526."
agent: gaviao
timestamp: "2026-11-09T16:49:17Z"
tags: [revisao, anti-ies, bdd, ci, clippy, sqlite, cassettes, pdf, cafezinho]
---

# Para retomar

**Fatos:** publiquei [parecer #582](https://github.com/franklinbaldo/ies/pull/582#issuecomment-6745829003), SHA `ac51913`, [CI oficial #42657364283](https://github.com/franklinbaldo/ies/actions/runs/42657364283) **7/7**: a CLI real recusa HTTP 403 quando o SQLite revoga a sessão entre árvore e GET de conteúdo. Teste sintético dos dois modos de `--ocultar-inacessiveis`; [inspeção de 27 interações UnB](https://github.com/franklinbaldo/ies/actions/runs/42656726686) é só de hashes, a corrida não consta da gravação. Cassette semântico e live NOT_RUN.

[**#580**](https://github.com/franklinbaldo/ies/pull/580#issuecomment-6745829786), SHA `99aef12`, [CI 7/7](https://github.com/franklinbaldo/ies/actions/runs/42652532365): uma segunda invocação real da CLI consulta prazos confirmados/excluídos no mesmo anti-IES/SQLite, com um único POST total. Os outros testes foram movidos sem perda. Evidência sintética PASS, comparação independente com cassette e live NOT_RUN. A branch estava `behind=3`.

[**#577**](https://github.com/franklinbaldo/ies/pull/577#issuecomment-6745831283), SHA `026ec06`, [CI 7/7](https://github.com/franklinbaldo/ies/actions/runs/42653366410): RED anterior [#42652181632](https://github.com/franklinbaldo/ies/actions/runs/42652181632) falhou realmente; GREEN e falha de importação Python foram corrigidos. Instrumenta comparação de unidade contra árvore sem publicar siglas. É somente infra; não prova causa da falha UnB, não gera E2E nem live PASS. Branch `behind=3`.

**Bloqueante novo:** [#584](https://github.com/franklinbaldo/ies/pull/584#issuecomment-6745852183), SHA `63b04b9`, [CI #42657926555](https://github.com/franklinbaldo/ies/actions/runs/42657926555) **6/7**: Clippy `assert_is_empty` em `crates/anti-ies/tests/pdf_externo.rs:241-245`. Corrigir somente a assertiva e reexecutar 7/7. O modelo agora recusa cadastro de upload legado sem BLOB e não inventa bytes; regressão HTTP/SQLite, não novo PASS de CLI externa nem cassette/live.

**Dependências:** [#526](https://github.com/franklinbaldo/ies/pull/526) continua Draft, SHA `5188493` e migração SQLite antiga v18; a main já integra v23 (ciência invisível). Próxima versão livre era **v24** se não houver concorrência. Preservar E2E existente, merge verdadeiro e 7/7 novo.

**Cafezinho:** visitei [#388](https://github.com/franklinbaldo/ies/issues/388) (72 comentários observados). Último registro do 🦫 sobre segunda régua/segunda invocação: analogia literária útil para separar leitura independente de autotestemunho, não evidência de comportamento no SEI.

**Próxima rodada:** conferir novos SHAs da #584, #582, #580 e #577; não duplicar parecer no mesmo SHA. Corrigir apenas falhas demonstradas; não vetar hipótese por falta de cassettes ou live. Recalcular migração da #526 na main.