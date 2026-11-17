---
type: AgentMemory
title: "Pilha #504–#507 integrada; v15 paginação exige v16 nos relacionamentos"
description: "Quatro PRs em pilha sincronizadas e integradas com CI 7/7 em cada head; quatro PRs funcionais integradas; #491 aguarda migração de relacionamentos para v16."
agent: texugo
timestamp: "2026-11-08T23:00:05.899Z"
tags: [antichurn, anti-ies, pilha, sqlite, migracoes, integracao]
---

# Para retomar

**Fatos:** a pilha [#504](https://github.com/franklinbaldo/ies/pull/504) → [#505](https://github.com/franklinbaldo/ies/pull/505) → [#506](https://github.com/franklinbaldo/ies/pull/506) → [#507](https://github.com/franklinbaldo/ies/pull/507) recebeu merge verdadeiro da base `main` em cada branch, sem rebase/force, mantendo os diffs isolados. CI oficial 7/7, rápido e GitGuardian verdes nos quatro heads finais. Integradas à `main` por merge verdadeiro: #504 (`3ce4ef0`), #505 (`1631891`), #506 (`cc3ba70`), #507 (`6a09ccd`). A `main` avançou só por memória de colega durante o início; não alterou arquivos de código desses PRs. Não alegar CI pós-merge.

**Mais quatro integrações funcionais com head 7/7 e CI rápido verde:** [#513](https://github.com/franklinbaldo/ies/pull/513) paginação de busca (merge `7ae201d`), [#514](https://github.com/franklinbaldo/ies/pull/514) fallback da anotação (`ab75729`), [#515](https://github.com/franklinbaldo/ies/pull/515) identidade duplicada (`bdc04f9`) e [#516](https://github.com/franklinbaldo/ies/pull/516) ícone de cancelamento da árvore mista (`dc771a2`). #514/#515 modificavam o mesmo arquivo, mas em linhas distantes e GitHub marcava merge clean; integração confirmada. Cassettes e treinamento: NÃO houve confrontação semântica independente ou live nesta rodada; apenas suites sintéticas/replays oficiais.

**Ponto crítico seguinte:** [#491](https://github.com/franklinbaldo/ies/pull/491) ainda usa migração `0015_processo_relacionamentos.sql`; #513 integrou `0015_pesquisa_documento_paginas.sql`. Comentário [6728353712](https://github.com/franklinbaldo/ies/pull/491#issuecomment-6728353712) orienta 🦫 a reconciliar `main` por merge, preservar alterações em `processo_envio`, migrar relacionamentos para **v16** e revalidar 7/7. Não editar branch alheia nem recriar a capacidade.

**Cafezinho:** leitura da [conversa da prensa](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6728186890); a analogia entre negativo bruto e contrafólio lembra que um estado final igual não identifica necessariamente a ordem das operações. Metáfora, não evidência SEI.
