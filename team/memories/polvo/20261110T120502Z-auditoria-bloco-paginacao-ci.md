---
type: AgentMemory
title: "Blocos: ausência, vazio e paginação não são a mesma prova"
description: "Auditoria #591 revelou notas desatualizadas sobre POST de página 2 interna; PR #664 corrigiu decisão/evidência com CI oficial 7/7, sem paridade cassette/live."
agent: polvo
timestamp: "2026-11-10T12:05:02.243Z"
tags: [blocos, paginacao, anti-ies, sqlite, auditoria, cassette-first, ci, documentacao]
---

# Ponto de retomada

- **Fato:** a [auditoria #591](https://github.com/franklinbaldo/ies/issues/591#issuecomment-6766613341) comparou leitor, CLI, modelo SQLite, anti-IES, testes E2E e cards UnB/GO. A [PR #664](https://github.com/franklinbaldo/ies/pull/664) atualiza só `docs/decisoes/bloco-listar.md` e `contracts/evidencias/bloco-listar.md`: as versões anteriores diziam que não havia POST, mas BDD-256-02 já navega a segunda página **interna** com 2 GET + 1 POST e confere faixa e total.
- **Gate falsificador:** o [CI #42811534241](https://github.com/franklinbaldo/ies/actions/runs/42811534241) passou seis jobs e falhou em `check_evidence`: o registro com `unresolved_steps: 3` precisa conter literalmente `PENDENTE_DE_CASSETTE`. Reintroduzi o marcador (sem fingir evidência) no head [`6b03b26`](https://github.com/franklinbaldo/ies/commit/6b03b26376c83be0913f41e061741691c36d5bf2). O [CI oficial #42812092859](https://github.com/franklinbaldo/ies/actions/runs/42812092859) terminou **7/7 success** e o rápido #42812079585 passou. PR Ready, aberta, sem merge. A `main` avançou 2 memórias antes desta, de modo que integrador 🐘/🦡 deve conciliar e revalidar.
- **Três níveis de prova:** testes CLI real ↔ anti-IES ↔ SQLite existem e passaram nos gates; os três **cards** originais GO interno (57), UnB listagem (25) e GO listagem (299) foram lidos, mas sem rehash ZIP ou confronto semântico novo (**NOT_RUN**); treinamento/live **NOT_RUN**, nenhuma escrita externa no SEI.
- **Modelo de observação:** `None` no SQLite significa página não observada, não `total=0` comprovado. O [Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6766672756) explorou essa distinção e sua conexão com o POST já implementado.
- **Retomar:** revisão independente da #664 antes de integrar; não duplicar a fatia da 🦊 na #638 nem converter a #381, ainda sem contrato BDD aprovado, em feature prematura.
