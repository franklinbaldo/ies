---
type: AgentMemory
title: "BDD-264-01: atribuição reconciliada após criação de processo"
description: "PR #521 incorporou main por merge verdadeiro, preservou criação v17 e deslocou atribuição a SQLite v18; CI novo 7/7."
agent: raposa
timestamp: "2026-11-09T01:15:17Z"
tags: [cli, atribuicao, anti-ies, sqlite, integracao]
---

# Retomar

Fato: a [PR #521](https://github.com/franklinbaldo/ies/pull/521) recebeu o merge verdadeiro [`ea0c268`](https://github.com/franklinbaldo/ies/commit/ea0c268a752907e70defdac9c641e61c9567b41b) com `main`: migração `0017_tipo_processo_simulado.sql` da Coruja permanece, e `0018_processo_atribuicao.sql` tornou-se a próxima; duas rotas e módulos coexistem. [CI oficial 7/7](https://github.com/franklinbaldo/ies/actions/runs/42544273384) e [rápido PASS](https://github.com/franklinbaldo/ies/actions/runs/42544225717) no SHA reconciliado; `behind=0` na consulta. [Handoff na #264](https://github.com/franklinbaldo/ies/issues/264#issuecomment-6729909699).

Não atribuir paridade com cassettes ou live ao GREEN sintético: apenas hashes GO/UnB estavam comprovados na inspeção anterior. Revisão 🦅 e integração 🐘/🦡 precedem a próxima fatia BDD-264-02/03; o POST público continua fechado. O [Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388) continua usando a prova da folha `02` como metáfora de observações discriminantes, não fonte de fatos sobre o SEI.
