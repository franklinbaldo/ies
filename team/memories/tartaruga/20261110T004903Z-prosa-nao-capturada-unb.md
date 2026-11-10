---
type: AgentMemory
title: "Cassette UnB: o histórico não autoriza inventar corpo de versão"
description: "PR #621 prova em RED que o replay histórico atribuía prosa não capturada; GREEN deixa somente placeholder estrutural, CI oficial 7/7."
agent: tartaruga
timestamp: "2026-11-10T00:49:03.539Z"
tags: [anti-ies, cassette-first, bdd60, proveniencia, historico, tdd]
---

# Para retomar

**Fato:** a [PR #618](https://github.com/franklinbaldo/ies/pull/618) já está integrada (merge `a245edb8`). Na nova [PR #621](https://github.com/franklinbaldo/ies/pull/621), o replay do histórico original UnB usava `<p>Conteúdo não observado nesta versão.</p>` para corpos que a gravação **não contém**. O [RED no CI #42730335296](https://github.com/franklinbaldo/ies/actions/runs/42730335296) falhou exatamente na asserção contra essa prosa fictícia. O GREEN troca por `<p></p>`, apenas estrutura de um campo obrigatório do SQLite de teste; não transforma a versão sem corpo em versão observada.

**Provas:** [CI completo #42730586574](https://github.com/franklinbaldo/ies/actions/runs/42730586574) 7/7 e [rápido #42730579641](https://github.com/franklinbaldo/ies/actions/runs/42730579641) PASS no head `034b2b00d7dc8de7c31a749775cc37420481b275`. O replay original UnB (card SHA-verificado) confronta **metadados e oferta de histórico parcialmente**. O RED foi da fixture/proveniência, **não** de produção. Nenhum E2E novo do binário; regressões sintéticas existentes passaram. Live/REA NOT_RUN. Sem credenciais, produção, subcódigos ou TLS enfraquecido.

**Para a próxima rodada:** [handoff na issue #60](https://github.com/franklinbaldo/ies/issues/60#issuecomment-6755632422), PR aberta Ready, revisão 🦅 e integração 🐘/🦡; conferir a `main` atual, incorporar a memória por merge verdadeiro caso a PR fique atrás e revalidar gates no novo SHA. O [Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388) foi revisitado; permanece valiosa a distinção da equipe entre estado observado, hipótese de modelo e efeito comprovado.
