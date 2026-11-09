---
type: AgentMemory
title: "POST local do anti-IES visto pela CLI real"
description: "A adição sintética de relacionamento muda SQLite e a prévia da CLI observa a mesma aresta; confirmação externa continua bloqueada."
agent: castor
timestamp: "2026-11-09T02:16:55Z"
tags: [bdd, relacionamentos, anti-ies, sqlite, rust, cassette]
---

# Retomar a junta

Na [PR #528](https://github.com/franklinbaldo/ies/pull/528), o anti-IES passou a aceitar POST de adição **somente em simulação**, com identidade NUP/id tipada, SQLite v16 e GET bilateral. Um teste executa POST no HTTP loopback e depois o **binário IES real por TLS** sobre o mesmo banco, observando `sem_efeito=true` sem segundo POST mutador. [RED #42548585208](https://github.com/franklinbaldo/ies/actions/runs/42548585208): dois testes falharam antes do código. [CI GREEN #42549657274](https://github.com/franklinbaldo/ies/actions/runs/42549657274): 7/7 no head `b27d29de`, Ready, ainda não integrado.

[Inspeção PEN1](https://github.com/franklinbaldo/ies/actions/runs/42548569540): dois cards, 53 interações, hashes íntegros; não há POST direto de adição. Portanto prova **sintética parcial**, não confronto semântico de cassette nem treinamento. `--confirmar` da CLI continua bloqueado antes da rede. Continuar BDD-258-02/03/04 sem repetir o cenário de prévia.

A metáfora da 🦊 sobre projeções iguais de sequências distintas no [Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6729399043) recorda que reler um estado não reconstitui sozinho a causalidade; é uma intuição, não evidência física ou do SEI.
