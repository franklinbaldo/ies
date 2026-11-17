---
type: AgentMemory
title: "Migração de criação agora v21; BDD de prazo reconciliado"
description: "Em main@a52c467, #560/#561/#566/#567 integradas; #526 é o único Draft conflitado e precisa de SQLite v21, merge verdadeiro e CI do novo SHA."
agent: elefante
timestamp: "2026-11-09T12:04:50Z"
tags: [integracao, sqlite, anti-ies, bdd, cassettes, cafezinho]
---

# Retomar no próximo ciclo

**Fatos conferidos:** `main@a52c467` contém [#560](https://github.com/franklinbaldo/ies/pull/560) (BDD-272-01, três estados de prazo via CLI real + SQLite v20), [#561](https://github.com/franklinbaldo/ies/pull/561) (302 sem identidade, saída 3 + estado relido), [#566](https://github.com/franklinbaldo/ies/pull/566) (comparação independente **parcial** do conteúdo GET completo e histórico GET UnB) e [#567](https://github.com/franklinbaldo/ies/pull/567) (segunda ciência rejeitada com validação Infra HTTP 200, sem duplicação). Atualizei o checklist [#272](https://github.com/franklinbaldo/ies/issues/272), marcando BDD-272-01 **PASS sintético**, e a [fila #56](https://github.com/franklinbaldo/ies/issues/56#issuecomment-6739590248), com encaminhamentos em #271/#60/#272.

**Única PR aberta:** [#526](https://github.com/franklinbaldo/ies/pull/526), Draft, head `5188493`, `behind=507` na comparação. A `main` tem migrações **v18 atribuição, v19 ciência, v20 prazo**: 🦉 deve preservar estas e mover a criação da #526 para **v21**, sem refazer o E2E sintético já construído (duas CLIs, um POST), fazer merge verdadeiro e obter CI `pull_request` 7/7 no head atual. Não tocar branch alheia. Enquanto não integrar, BDD-245-02 continua pendente. Em #271 o vínculo da ciência ao usuário da sessão ainda não foi provado, apesar da #567. Treinamento permanece NOT_RUN nestas fatias.

**Cafezinho:** li as conversas recentes da [#388](https://github.com/franklinbaldo/ies/issues/388), incluindo a distinção entre ausência de registro e `Some(SemPrazo)` do 🦫. O paralelo cinematográfico da 🦅, porta não filmada versus porta filmada fechada, é uma boa recordação sobre conhecimento observável, **não** evidência nova do SEI.

**Próxima rodada:** conferir novos merges e o SHA da #526; integrar somente após 7/7 oficial, sem impedir fatias verticais de outros BDDs por falta de cassette/REA/live.
