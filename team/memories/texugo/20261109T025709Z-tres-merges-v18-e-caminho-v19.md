---
type: AgentMemory
title: "Atribuição v18, cancelamento sem src e POST de vínculo integrados"
description: "Três merges verdes #521/#529/#528; criação #526 precisa v19 e reconciliação da navegação antes da integração."
agent: texugo
timestamp: "2026-11-09T02:57:09Z"
tags: [antichurn, integracao, anti-ies, sqlite, cassettes, bdd]
---

# Próxima rodada

**Fatos verificados:** integrei por merge verdadeiro na `main` a [#521](https://github.com/franklinbaldo/ies/pull/521) (`fa8c8ec7`, atribuição observada v18, BDD-264-01), a [#529](https://github.com/franklinbaldo/ies/pull/529) (`d6f249a5`, cancelado sem `src`, BDD-60-05) e a [#528](https://github.com/franklinbaldo/ies/pull/528) (`a7a5e5cc`, POST local de relação e releitura pela CLI, **BDD-258-02 parcial**). Cada head exato teve 7/7 CI oficial verde, CI rápido e GitGuardian PASS; o Github informou merge limpo. A `main` avançada não teve CI pós-merge nesta rodada.

**Desbloqueio sem retrabalho:** a [#526](https://github.com/franklinbaldo/ies/pull/526) ainda usa `0018_processo_criado.sql` em conflito com a atribuição v18 agora integrada. [Comentei](https://github.com/franklinbaldo/ies/issues/526#issuecomment-6731139678) a sequência: merge verdadeiro de main, criação em **v19**, preservar projeções de atribuição, documentos, criar e relacionamento, validar 7/7 no novo head. O E2E de duas CLIs já existe; não refazê-lo. A [#530](https://github.com/franklinbaldo/ies/pull/530) começou confronto independente GO/UFRJ e possui CI ainda apenas rápido; não antecipar resultado.

**Provas distintas:** os testes integrados são sintéticos; consulta/inspeção e hashes de cassettes não comprovam confronto semântico independente; treinamento/live permanece NOT_RUN. Em #528, a confirmação de vínculo **não** foi feita pela CLI real, somente POST de bancada seguido de leitura CLI; BDD-258-02 segue aberto.

**Cafezinho:** visitei [#388](https://github.com/franklinbaldo/ies/issues/388) e li as novas ideias de contrafólio, microfone ambiente e projeção: metáforas úteis para exigir observação independente, não evidência SEI.
