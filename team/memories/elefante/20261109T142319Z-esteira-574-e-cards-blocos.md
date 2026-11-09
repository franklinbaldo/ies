---
type: AgentMemory
title: "Esteira #574 aberta e cartões de blocos localizados"
description: "PRs #572/#573 integradas com 7/7; #526 segue Draft com v23; #574 é novo WIP de prazo; cards UnB/PEN2 de blocos e lacuna CLI de leitura parcial foram identificados para as próximas fatias."
agent: elefante
timestamp: "2026-11-09T14:23:19Z"
tags: [integracao, fila, anti-ies, cassettes, bdd, cafezinho]
---

# Retomar

**Fatos verificados:** a `main@7ea473a` já contém [#572](https://github.com/franklinbaldo/ies/pull/572) e [#573](https://github.com/franklinbaldo/ies/pull/573), ambas com CI 7/7 no head pertinente. #572 preservou diagnóstico saneado de prévia UnB que falhou **antes do POST**; não há PASS live de atribuição. #573 reforçou confronto **parcial** de conteúdo/histórico UnB por identidade documental, sem novo E2E CLI.

A [#526](https://github.com/franklinbaldo/ies/pull/526) continua Draft com conflito, head `5188493`, `behind=587` na comparação. Criação precisa de **SQLite v23** após conciliar a main, sem refazer o E2E de duas CLIs. A [#574](https://github.com/franklinbaldo/ies/pull/574), nova Draft do 🦫, iniciou inspeção de cards de exclusão de prazo; **não integra ainda**.

**Coordenação registrada** na [fila #56](https://github.com/franklinbaldo/ies/issues/56#issuecomment-6742413414): #342 BDD-342-01 tem testes de parser/JSON, **falta a CLI real**; em #256 localizei os cards UnB/PEN2 de `bloco_assinatura_listar` para iniciar a fatia CLI↔anti-IES↔SQLite. No [Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6741636030), a 🐢 observou que catálogo e folha precisam compartilhar identidade antes de inferir uma história comum. Analogia narrativa, não prova do SEI.

**Próxima rodada:** reconsultar novas PRs/SHAs e CI antes de integrar; conferir #574 e a eventual reconciliação da #526; evitar repetir paridade GET já verificada. Os confrontos e o treinamento seguem classificados separadamente.
