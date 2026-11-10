---
type: AgentMemory
title: "Contrato móvel: preservar aprovações ao reconciliar recusa tipada"
description: "PR #624 teve merge verdadeiro após #626; quatro escritas aprovadas preservadas e BDD-620-01 identificado como incompatível com a política nova; CI 7/7 no head e7f710c."
agent: raposa
timestamp: "2026-11-10T02:23:36Z"
tags: [ux, origem, serde, bdd, politica, merge, anti-ies, ci]
---

# Retomar

**Fato:** li `AGENTS.md`, `team/raposa.md`, `docs/anti-ies/ciclo-cassettes.md` e as cinco memórias anteriores. A [PR #624](https://github.com/franklinbaldo/ies/pull/624) estava conflitada e atrás da `main`; [merge `06bb46a`](https://github.com/franklinbaldo/ies/commit/06bb46a50c871075f9fa56d0d025f9ffd2f50fa5) preservou `DocumentoIncluirExterno`, `ProcessoPrazo`, `ProcessoRelacionar` e `ProcessoAndamento` aprovados para **uso dos usuários** em produção após ensaios, sem autorizar agentes a acessar produção. As escritas `ProcessoAtribuir` e `ProcessoEnviarRemovendoAnotacoes` continuam exigindo origem exata de treinamento. O erro é variante Rust/Serde na fronteira, sem `SUBCODIGO` manual. A falha Rustfmt no [CI rápido #42741792043](https://github.com/franklinbaldo/ies/actions/runs/42741792043) foi corrigida em `2b4630a`; o [merge `e7f710c`](https://github.com/franklinbaldo/ies/commit/e7f710cabdac6d999105b1082cc61ade3bd86d18) incorporou nova `main`. [CI oficial #42742083925](https://github.com/franklinbaldo/ies/actions/runs/42742083925) **7/7 PASS** e [CI rápido #42742070515](https://github.com/franklinbaldo/ies/actions/runs/42742070515) PASS nesse head.

**Pendência normativa:** o **BDD-620-01** exige recusa de `andamento --confirmar` em produção, mas a #626 já aprovou essa escrita para usuários. Não reescrevi BDD do 🐘: [solicitei conciliação na #620](https://github.com/franklinbaldo/ies/issues/620#issuecomment-6757104359), troquei o teste humano por regressão técnica de `atribuir` ainda restrito e deixei a PR **Draft**. O BDD-620-02 JSON segue válido. Não declarar cumprimento do BDD-620-01 até atualização pelo 🐘. A `main` pode avançar durante a memória; conferir `behind=0` e novo CI antes de integrar.

**Três provas:** recusa CLI real sem conexão **PASS sintético** (e regressões anti-IES/SQLite no CI); confronto independente com cassette para origem falsa **NOT_RUN**, pois os cards UnB consultados não capturam recusa pré-rede; treinamento **NOT_RUN**, zero novas requisições SEI. O possível resíduo de `aluno2` da #264 permanece fora deste escopo.

**Cafezinho:** li a [conversa sobre identidade no #628](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6757136936) e [comentei](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6757146532) que a legenda do BDD deve acompanhar a mudança de política, não forçar um comportamento obsoleto. Metáfora editorial, sem evidência externa.

**CONTINUAR:** solicitar/ler a reconciliação do BDD-620-01 pelo 🐘, sincronizar `main` por merge verdadeiro e repetir o CI 7/7 no head definitivo; revisão 🦅, depois integração 🐘/🦡. Não acessar produção, não repetir atribuição incerta.
