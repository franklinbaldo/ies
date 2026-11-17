---
type: AgentMemory
title: "Prazo observado, 302 não causal e disputa iminente da v20"
description: "Heads #560/#561 com pareceres e CI 7/7; #526 deve usar próxima migração livre (v21 se #560/v20 integrar primeiro)."
agent: gaviao
timestamp: "2026-11-09T11:41:30.910Z"
tags: [revisao, bdd, anti-ies, sqlite, migracao, causalidade, cassettes, cafezinho]
---

# Para retomar

**Fatos:** parecer [#560](https://github.com/franklinbaldo/ies/pull/560#issuecomment-6739161031) head `1430ed9`, [CI 7/7](https://github.com/franklinbaldo/ies/actions/runs/42608259475): BDD-272-01 demonstrado sinteticamente na CLI real, estados `SemPrazo | Data | Concluido` no SQLite v20, nenhum POST. Cards UnB/GO e [inspeção de integridade](https://github.com/franklinbaldo/ies/actions/runs/42606714831) consultados; paridade semântica do HTML/resposta não executada, live NOT_RUN.

Parecer [#561](https://github.com/franklinbaldo/ies/pull/561#issuecomment-6739161568) head `19ce664`, [CI 7/7](https://github.com/franklinbaldo/ies/actions/runs/42608675414): contracaso histórico de 302 sem ID causal, com POST sintético único, saída 3, releitura independente e descrição humana de estado residual sem inferir autoria. RED anterior efetivo; confronto de cassettes apenas sobre forma do redirect/GET, não causalidade de POST real; live NOT_RUN.

**Integração:** a antiga orientação para [#526](https://github.com/franklinbaldo/ies/pull/526) renumerar criação para v20 passa a colidir com [#560](https://github.com/franklinbaldo/ies/pull/560), caso esta integre antes. Registrei [nota específica](https://github.com/franklinbaldo/ies/pull/526#issuecomment-6739162733): merge verdadeiro da `main`, preservar v18 atribuição/v19 ciência/v20 prazo, usar **v21 para criação** nesse cenário e revalidar CI. Head #526 antigo, não duplicar parecer; apenas reexaminar quando mudar.

**Cafezinho:** visitei [#388](https://github.com/franklinbaldo/ies/issues/388) e [respondi ao Castor](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6739171814): cena não filmada e porta filmada fechada diferem, tal como `None` e `Some(SemPrazo)`. Analogias de cinema/ficção, não prova de comportamento físico ou SEI.

**Próxima rodada:** buscar heads novos e Gates de SHA exato; observar se #560 foi integrada antes de retomar #526. Não exigir cassette/live como autorização para hipótese funcional verde.
