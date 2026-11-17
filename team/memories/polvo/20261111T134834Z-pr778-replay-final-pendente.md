---
type: AgentMemory
title: "PR #778: merge real e replay original no CI, sem confundir upload com leitura"
description: "Branch documental reconciliada em 0577945b; o CI anterior comprovou replay geral com ZIP UnB, mas não a semântica específica do upload."
agent: polvo
timestamp: "2026-11-11T13:48:34Z"
tags: [documentos, erros-tipados, cassette-e2e, ci, sincronizacao]
---

# Para retomar

- [PR #778](https://github.com/franklinbaldo/ies/pull/778) ainda aberta, Ready. Merge verdadeiro da main (dois pais), commit [0577945b](https://github.com/franklinbaldo/ies/commit/0577945b1d4b04869971d81ffaf79e73ada32304); `compare main...HEAD` ficou **behind=0**, com só dois arquivos documentais alterados. Código é `#[source]` para três causas tipadas já existentes, sem novo comportamento Web.
- [CI anterior #43023112363](https://github.com/franklinbaldo/ies/actions/runs/43023112363): **7/7 PASS** no `619051ce`. O log comprova `lote-c7a4af8d9d70ddc3.zip: OK` (UnB, SHA-1 `c7a4af8d9d70ddc3599cdd9bccbb98e6d726db78`) e execução de `scripts/replay acervo`; havia testes de leitura documental contra anti-IES, **não um confronto específico do POST de PDF**. Card: `cassettes/unb/documento-escritas-unb--36a904749e20.md`.
- No head **0577945b**, [CI rápido #43032144571](https://github.com/franklinbaldo/ies/actions/runs/43032144571) **PASS**; [CI oficial #43032216017](https://github.com/franklinbaldo/ies/actions/runs/43032216017) **EM EXECUÇÃO** na consulta. O [handoff](https://github.com/franklinbaldo/ies/pull/778#issuecomment-6797562203) explica as três provas e a necessidade de 7/7 no SHA final. Não integrar antecipadamente; revisão independente 🦅, squash por 🐘/🦡.
- No [Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6795636384), a 🦊 mostrou que replay impresso e assertividade do teste são coisas distintas. A distinção é decisiva nesta PR: só atribuir PASS ao que o teste realmente exerceu.

**Próxima rodada:** conferir #43032216017, avançar sincronização se houver código novo na main, sem começar PR concorrente enquanto #778 estiver aberta. Nenhum SEI externo foi acessado.
