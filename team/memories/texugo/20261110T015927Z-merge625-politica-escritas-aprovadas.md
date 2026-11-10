---
type: AgentMemory
title: "Merge da colisão PDF/interno e conflito com aprovação de escritas"
description: "Integrei #625 (CI 7/7 e rápido no head); #624 e #622 precisam preservar as aprovações de produção para usuários introduzidas pela #626, sem reduzir a guarda das operações não aprovadas."
agent: texugo
timestamp: "2026-11-10T01:59:27Z"
tags: [antichurn, integracao, bdd, autorizacao, cassettes, ci]
---

# Para retomar

**Fatos:** [#625](https://github.com/franklinbaldo/ies/pull/625) integrada na `main` por merge `cfe4efb9`, head `2d79e8b1`, [CI oficial 42739168404](https://github.com/franklinbaldo/ies/actions/runs/42739168404) **7/7** e CI rápido 42739153003 verde; BDD-60-05 testa recusa da CLI real contra anti-IES/SQLite quando PDF e documento interno compartilham número. É **PASS sintético**; as cassettes UnB tiveram integridade validada, mas **colisão específica e live NOT_RUN**. Não houve CI pós-merge.

**Conflito material:** [#626](https://github.com/franklinbaldo/ies/pull/626) já integrada aprovou para **uso dos usuários** em produção também `ProcessoRelacionar` e `ProcessoAndamento` após ensaio UnB. [#624](https://github.com/franklinbaldo/ies/pull/624) está desatualizada/conflitante e sua lista `aprovada_em_treinamento()` antiga ainda contém apenas PDF externo e prazo; isso reverteria as duas aprovações. [Orientei reconciliação cirúrgica](https://github.com/franklinbaldo/ies/pull/624#issuecomment-6756789157), preservando a causa Serde, as quatro operações aprovadas e a recusa pré-rede para atribuição/envio. O CI 7/7 no head antigo não aprova essa integração.

A [#622](https://github.com/franklinbaldo/ies/pull/622) tem CI 7/7 no head `dc9c3a1`, mas traz contrato anterior à aprovação live; [orientei merge verdadeiro](https://github.com/franklinbaldo/ies/pull/622#issuecomment-6756789587) preservando a política nova e o E2E de 503 pós-POST. [#627](https://github.com/franklinbaldo/ies/pull/627) é Draft RED; [#526](https://github.com/franklinbaldo/ies/pull/526) segue Draft e conflitante. Não editar branches alheias nem integrar enquanto os novos heads não forem validados.

No [Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6756225195), a 🦊 observou que texto de saída deve derivar da causa tipada; o exemplo ajuda a separar aprovação da operação (domínio) da apresentação do erro (fronteira). É analogia, não prova de SEI.

Nenhuma requisição a SEI, escrita em produção, novo subcódigo ou gate nesta rodada.
