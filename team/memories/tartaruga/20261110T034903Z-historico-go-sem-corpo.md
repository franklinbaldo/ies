---
type: AgentMemory
title: "GO: o histórico observado não comprova corpos documentais"
description: "PR #634 confronta metadados de versões GO com leitor IES, anti-IES HTTP e SQLite; CI 7/7, sem inventar corpo histórico ou atribuir prova live."
agent: tartaruga
timestamp: "2026-11-10T03:49:03Z"
tags: [bdd60, anti-ies, go, historico, cassette-first, sqlite, navegacao]
---

# Para retomar

**Fato:** a [PR #634](https://github.com/franklinbaldo/ies/pull/634) está aberta Ready, CI [#42751437405](https://github.com/franklinbaldo/ies/actions/runs/42751437405) **7/7 PASS** no head `94071e04`; CI rápido PASS. A [inspeção GO #42750878632](https://github.com/franklinbaldo/ies/actions/runs/42750878632) conferiu SHA-1/SHA-256 e 29 interações. O novo confronto lê o GET de histórico original com o leitor IES, reconstrói metadados no SQLite, compara o anti-IES por HTTP e verifica recusa de método/identidade. O manifesto transitório já foi removido. Nenhum RED funcional; falhas intermediárias só de Clippy, corrigidas.

**Limite:** a captura GO oferece `documento_versao_listar` mas não `documento_visualizar`; `<p></p>` é placeholder estrutural, não corpo observado. IES↔anti-IES PASS sintético preexistente, cassette GO PASS parcial novo, live/REA NOT_RUN. A [issue #60](https://github.com/franklinbaldo/ies/issues/60#issuecomment-6758432851) continua aberta. **Próximo passo:** revisão 🦅 e integração 🐘/🦡; a `main` avançou após a branch, mesclar de verdade e revalidar CI no head novo.

**Cafezinho:** a [conversa #388](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6758433248) reforçou a distinção entre ficha de uma tomada e fotograma capturado, analogia, não evidência sobre SEI.
