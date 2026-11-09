---
type: AgentMemory
title: "Pasta pendente é hipótese tipada, não documento ausente"
description: "BDD-342-01 ganhou E2E CLI real com SQLite v24; dois erros de gate eram expectativa antiga de schema e doc_markdown, corrigidos sem flexibilizar verificações."
agent: castor
timestamp: "2026-11-09T17:41:40Z"
tags: [bdd, anti-ies, processo, sqlite, clippy, cassette-first]
---

# Para a próxima rodada

**Fato:** a [PR #587](https://github.com/franklinbaldo/ies/pull/587) exerce `ies --json processo NUP` na CLI real com pasta pendente modelada em `PastaPendente`/SQLite v24. Ela devolve `documentos_incluidos:false` e aviso, sem afirmar lista vazia conhecida; o [CI completo #42666409983](https://github.com/franklinbaldo/ies/actions/runs/42666409983) passou 7/7 no head `d29386d8d`. A memória não substitui revisão nem merge.

**Aprendizado:** evoluir um esquema derivado exige atualizar expectativas explícitas em testes transversais: `retornos_cassettes` ainda exigia v23. O Clippy também exige marcar identificadores técnicos entre crases nas docstrings; corrigir as duas ocorrências, não silenciar o lint.

**Limite:** as duas cassettes UnB inspecionadas somam 498 interações com hashes íntegros, mas não capturam a falha sintética. IES↔anti-IES↔SQLite PASS sintético; confronto semântico independente e live NOT_RUN. A fixture não cobre ainda 19 de 20 documentos.

**Retomar:** revisão 🦅 e integração 🐘/🦡 da #587, observando se novas alterações da `main` exigem merge verdadeiro. No [Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388), a conversa sobre testemunhas diferentes permanece uma analogia útil: uma árvore declarada e a lista realmente lida não são a mesma prova.
