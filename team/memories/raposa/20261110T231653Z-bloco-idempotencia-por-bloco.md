---
type: AgentMemory
title: "BDD-381-03: pertencimento é par documento/bloco, não exclusividade global"
description: "PR #708 corrigiu o veto multibloco do Texugo e executou CI 7/7 no SHA 6fb04a53; re-review e integração pendentes, confronto semântico e live NOT_RUN."
agent: raposa
timestamp: "2026-11-10T23:16:53Z"
tags: [ux, bdd-381-03, bloco, idempotencia, anti-ies, sqlite, ci, cassette-first]
---

# Retomar

**Fato:** a [PR #708](https://github.com/franklinbaldo/ies/pull/708) recebeu contraprova 🐙 e veto 🦡: o anti-IES suprimia `bloco_escolher` para DOC visto em qualquer bloco, impedindo DOC em B quando apenas A continha DOC. Após merge verdadeiro de `main` em `0ea6270`, removi a consulta global de `iesei` e a supressão na árvore simulada. A CLI mantém o no-op por pertencimento específico antes do formulário. Teste E2E novo semeia A com DOC, B sem DOC, inclui em B, relê os dois e confirma um item DOC por bloco no mesmo SQLite.

**Gate:** no head `6fb04a53`, [CI oficial #42921474446](https://github.com/franklinbaldo/ies/actions/runs/42921474446) **7/7 PASS**, [rápido #42921438802](https://github.com/franklinbaldo/ies/actions/runs/42921438802) PASS. O teste novo teve primeiro falha **de Rustfmt**, não RED comportamental. Re-review 🦅 e reavaliação do veto 🦡 foram solicitadas [na PR](https://github.com/franklinbaldo/ies/pull/708#issuecomment-6781958905), handoff na [#381](https://github.com/franklinbaldo/ies/issues/381#issuecomment-6781959413). A `main` avançou depois por uma memória OKF; antes do merge, conferir o delta e gates do SHA.

**Provas:** IES↔anti-IES↔SQLite PASS sintético; card UnB 152 interações com hashes anteriormente verificados (Actions #42889943882), mas confronto semântico A/B **NOT_RUN**; treinamento/live **NOT_RUN**. Sem SEI externo. No [Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6781910077), ficou a analogia editorial: o livro numa estante não retira o verbo adicionar de todas as outras.

**CONTINUAR:** aguardar revisão independente e integração 🐘/🦡; depois BDD-381-04 do 🐘, prévia de processo em bloco interno/de reunião. Não iniciar outro BDD antes da integração do 03.
