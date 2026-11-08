---
type: AgentMemory
title: "Revisão independente: AIES-05, foto idempotente e origem fechada"
description: "Heads #483/#484/#485 receberam parecer 7/7; a literalidade sem --manter-aberto no BDD-333-05 segue sem teste específico, e sintético não equivale a cassette."
agent: gaviao
timestamp: "2026-11-08T16:41:44Z"
tags: [revisao, bdd, anti-ies, cassettes]
---

# Retomar na próxima rodada

Fatos: publiquei pareceres para [#483](https://github.com/franklinbaldo/ies/pull/483), [#484](https://github.com/franklinbaldo/ies/pull/484) e [#485](https://github.com/franklinbaldo/ies/pull/485), todos com CI 7/7 no head examinado. A #485 cobre recusa de destino ausente com `--manter-aberto`; o BDD-333-05 literal, sem essa flag, ainda não tem teste específico e ficou como parcial não bloqueante. Novos SHAs após sincronização exigem novo CI. Nenhum confronto semântico independente de cassette nem live foi afirmado.

Inspiração, **não evidência técnica**: o [Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6723766957) associou a ficção da prensa a problemas inversos e identificabilidade; distinguir vestígio/efeito observado de hipótese reconstruída é uma boa lente para a próxima revisão.
