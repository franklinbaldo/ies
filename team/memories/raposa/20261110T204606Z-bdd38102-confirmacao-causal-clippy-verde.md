---
type: AgentMemory
title: "BDD-381-02: a confirmação é 302 causal mais detalhe relido"
description: "PR #701 no SHA a955483: CI 7/7 após corrigir Clippy sem allow; CLI real e anti-IES persistem inclusão no SQLite, mas cassette UnB não comprova corpo do POST nem efeito. Revisão independente pendente."
agent: raposa
timestamp: "2026-11-10T20:46:06Z"
tags: [ux, bloco, bdd-381-02, anti-ies, sqlite, cassette-first, ci, cafezinho]
---

# Retomar na próxima rodada

Fato: a [PR #701](https://github.com/franklinbaldo/ies/pull/701) implementou a confirmação documental do BDD-381-02: um POST derivado de `frmBlocoEscolher`, aceite 302 causal e releitura independente do detalhe. A regressão executa `ies bloco incluir ... --confirmar` e uma segunda CLI `ies bloco BLOCO` sobre o mesmo SQLite. O [CI oficial #42897607950](https://github.com/franklinbaldo/ies/actions/runs/42897607950) passou **7/7** no SHA `a955483`; o [CI rápido #42897425376](https://github.com/franklinbaldo/ies/actions/runs/42897425376) também passou. Duas advertências `needless_pass_by_value` foram corrigidas com empréstimos; o RED comportamental inicial não chegou a executar o teste, embora o GREEN tenha sido observado.

Limite: [inspeção UnB #42889943882](https://github.com/franklinbaldo/ies/actions/runs/42889943882) validou os hashes de 152 interações e a sequência GET→POST 302→GET, **não** o corpo HTTP enviado ou a persistência do documento. E2E sintético **PASS**; confronto bilateral semântico e treinamento **NOT_RUN**. Não houve escrita externa.

Conversa no [Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6778796535): 302 é sinal da resposta, não prova do estado. **CONTINUAR:** revisão 🦅, integração 🐘/🦡 e só depois BDD-381-03 (idempotência e recusas), sem alterar o cenário escrito pelo 🐘.
