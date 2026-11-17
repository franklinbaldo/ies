---
type: AgentMemory
title: "BDD-60-05: recusa humana precisa respeitar NUP explícito"
description: "PR #697 testa pelo binário real a recusa humana de número duplicado ou alheio no NUP informado. Descartei variação sem --processo porque não era o BDD canônico; cassette UnB verificada em integridade, não no negativo."
agent: raposa
timestamp: "2026-11-10T19:22:36Z"
tags: [ux, bdd-60-05, documento, anti-ies, sqlite, cassette-first]
---

# Para retomar

A [PR #697](https://github.com/franklinbaldo/ies/pull/697) acrescenta verificação de mensagem humana para duas recusas já contratadas da [#60](https://github.com/franklinbaldo/ies/issues/60): número duplicado na árvore e documento pertencente a outro NUP. Os comandos passam pela CLI Rust real, TLS local, anti-IES e SQLite efêmero. **Status do CI: a conferir no SHA final**, sem reivindicar RED/GREEN antes do log.

Uma variante inicialmente proposta sem `--processo` foi retirada: não correspondia ao cenário BDD-60-05 expressamente escrito pelo 🐘. Se a ambiguidade da pesquisa global merecer uma fatia, primeiro pedir contrato e BDD ao 🐘, não transformar hipótese em cenário novo.

A [inspeção UnB no Actions](https://github.com/franklinbaldo/ies/actions/runs/42883498393) conferiu SHA-1/SHA-256 e 15 interações de `documento-numero-e-busca`. Isso é **PASS de integridade**, não evidência do negativo de duplicidade. Paridade bilateral e live continuam **NOT_RUN**. O [Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6775227625) lembrou que pesquisa flexível não transforma identidades distintas em equivalentes; aqui também não devemos adivinhar o contexto do documento.
