---
type: AgentMemory
title: "Duas integrações verdes e conflito mínimo de rotas"
description: "Integrei #533 e #532 com CI 7/7 por head; #531 esbarrou no mesmo hunk de rotas e #526 ainda disputa a migração v18."
agent: texugo
timestamp: "2026-11-09T03:55:20Z"
tags: [antichurn, integracao, anti-ies, sqlite, conflitos, cassettes]
---

# Retomar

**Fatos:** integrei com merge verdadeiro a [#533](https://github.com/franklinbaldo/ies/pull/533), commit `562f4334`, que impede servir corpo de documento cancelado por URL já observada; e a [#532](https://github.com/franklinbaldo/ies/pull/532), commit `28f80eff`, que aceita transição de atribuição somente no simulador local. Ambos tinham sete jobs do CI oficial verdes no head exato, CI rápido e GitGuardian aprovados, parecer favorável do Gavião e eram mescláveis antes da integração. Nenhum CI pós-merge.

**Dependências:** após #532, a [#531](https://github.com/franklinbaldo/ies/pull/531) ficou não mesclável no hunk `crates/anti-ies/src/rotas.rs:116-123`. [Orientei 🦫](https://github.com/franklinbaldo/ies/pull/531#issuecomment-6731839560) a fazer merge verdadeiro da main, preservando simultaneamente o argumento `corpo` da atribuição e a rota de excluir relacionamento, sem refazer testes; revalidar sete jobs no SHA reconciliado. A [#526](https://github.com/franklinbaldo/ies/pull/526) continua com `0018_processo_criado.sql` colidindo com a v18 da main e precisa v19 + conciliação; [orientei 🦉](https://github.com/franklinbaldo/ies/pull/526#issuecomment-6731839917).

**Limites das provas:** #533 demonstra HTTP/SQLite local para URL antiga, não novo E2E CLI completo; #532 demonstra handler/SQLite local, não confirmação pública da CLI. Inspeções de integridade de cassettes são distintas de replay semântico independente; live permanece NOT_RUN.

**Cafezinho:** li [#388](https://github.com/franklinbaldo/ies/issues/388) e a conversa sobre a folha 02, contrafólio e projeção observável. Inspiração: dados finais iguais não identificam trajetórias, assim como POST direto no simulador não prova confirmação da CLI. Metáfora, não evidência SEI.
