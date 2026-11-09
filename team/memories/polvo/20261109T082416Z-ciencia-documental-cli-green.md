---
type: AgentMemory
title: "Ciência documental: chamada por número e prova independente no SQLite"
description: "A PR #546 passou CI 7/7 após retirar dependência de NUP não registrado e medir 4 pedidos de prévia mais 11 da confirmação; atribuição ao usuário e confronto semântico ainda não provados."
agent: polvo
timestamp: "2026-11-09T08:24:16Z"
tags: [documento, ciencia, anti-ies, sqlite, bdd271, cassettes]
---

# Para a próxima rodada

- **Fato:** a [PR #546](https://github.com/franklinbaldo/ies/pull/546) está Ready no head `f22e565`; [CI #42585267913](https://github.com/franklinbaldo/ies/actions/runs/42585267913) **7/7 PASS** e [CI rápido #42585261857](https://github.com/franklinbaldo/ies/actions/runs/42585261857) PASS. O BDD-271-02 sintético percorre `ies documento ciente DOC` → TLS local → anti-IES → SQLite; prévia com 4 pedidos não escreve, confirmação usa 11 pedidos (inclui releitura independente e Lista de Ciências) e a atividade do JSON coincide com a linha SQL.
- **Aprendizado:** o RED original do [CI #42580178645](https://github.com/franklinbaldo/ies/actions/runs/42580178645) parecia ser falta de rede, mas o orçamento fixo de requests encobria a falha inicial: o `--processo` buscava NUP sem entrada em `sim_processo`, apesar de o documento já existir em `documento_simulado`. Remover a dependência e usar o número documental canônico resolveu, sem fixture adicional. Não confundir limite de requests na bancada com comportamento da CLI.
- **Limites:** cards UnB (39 interações) e UFRJ (38) consultados; replay dos 14 cards passa tecnicamente, mas não prova semântica do GET mutador nem atividade na resposta histórica. O modelo sintético ainda **não vincula a ciência à identidade do usuário da sessão**, logo a prova é parcial. SEI treinamento/live **NOT_RUN**; 0 chamadas e 0 escritas externas.
- **Retomada:** revisão 🦅 e integração 🐘/🦡 da PR, coordenando `0019_ciencia_documental.sql` com a PR #526 para não duplicar `user_version`. Resolver depois a identidade do usuário de forma observável, sem inventar dado de sessão; avançar confronto independente e testes de segunda ciência. A visita ao [Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388) não alterou critérios de prova.
