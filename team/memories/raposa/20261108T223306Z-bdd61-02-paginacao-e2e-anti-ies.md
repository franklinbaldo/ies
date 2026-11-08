---
type: AgentMemory
title: "BDD-61-02: página 2 reproduzida por CLI real e anti-IES"
description: "PR #513: SQLite efêmero v15, rotas de pesquisa e CLI E2E; CI 7/7 no head e hashes Go/UnB verificados, confronto semântico pendente."
agent: raposa
timestamp: "2026-11-08T22:33:06Z"
tags: [cli, bdd, anti-ies, sqlite, paginacao, cassette]
---

# Retomar com fatos

A [PR #513](https://github.com/franklinbaldo/ies/pull/513) implementou
**BDD-61-02**: `ies documento buscar 00050 --pagina 2 --json` usa IES real,
TLS local, anti-IES e SQLite efêmero `user_version=15` com páginas/linhas
ligadas aos documentos versionados já existentes. Na fixture sintética,
página 1 exibe 1–10 de 27 com 10 itens (3 requests); página 2, 11–20
com apenas 1 item de 9 linhas ausentes da faixa (4 requests). Nenhuma
linha da primeira é publicada na segunda; página sem observação não
ganha conteúdo fictício. Sem novos subcódigos ou contato SEI.

O [CI rápido #42530377179](https://github.com/franklinbaldo/ies/actions/runs/42530377179)
deu PASS, e o [CI oficial #42530397830](https://github.com/franklinbaldo/ies/actions/runs/42530397830)
foi **7/7 SUCCESS** no head `e0b539d`. Primeiro CI
[#42530183265](https://github.com/franklinbaldo/ies/actions/runs/42530183265)
comprovou os seis E2E da busca (incluindo dois de paginação), mas falhou
por três asserções legadas de `user_version=14`, ajustadas para 15.

Cassette-first: cards `pesquisa-paginas-go` e `pesquisa-paginas-unb`
lidos antes da fixture; [GitHub Actions #42529991607](https://github.com/franklinbaldo/ies/actions/runs/42529991607)
confirmou hashes de 2 ZIPs/cassettes e 20 interações HTTP.
A inspeção só demonstra integridade e estrutura, **não confronto independente
dos corpos HTML, equivalência SEI ou treinamento**. REA não executado.

**Próximo passo:** 🦅 revisar, 🐘/🦡 integrar a PR #513 e confrontar
independentemente corpos originais das cassettes. O commit desta memória,
por ser adição direta na `main`, pode exigir nova sincronização da PR
antes do merge, sem rebase/force-push. BDD-61-01 e 61-03 já integrados,
não refazer. A [conversa do Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388)
serve de lembrete informal: observação sintética e fonte real não se equivalem.
