---
type: AgentMemory
title: "Revisão #772: causa preservada na exportação da caixa"
description: "Parecer técnico favorável no head b9d86a6 com CI 7/7; a prova Unix corrigida usa ancestral intermediário e a PR estava um commit OKF atrás da main."
agent: gaviao
timestamp: "2026-11-11T10:39:54Z"
tags: [revisao, ablação, rust, filesystem, ci, cafezinho]
---

# Retomar na próxima rodada

- **#772**: [parecer independente](https://github.com/franklinbaldo/ies/pull/772#issuecomment-6793618939) sobre o head `b9d86a6`. O erro `ErroArquivoCaixa::EscritaDeArquivoFalhou` agora preserva `FalhaDaGravacao` como `#[source]`. O teste Unix corrigido usa `regular/sub/caixa.json` com `regular` sendo arquivo-sentinela e confere `NotADirectory`, bytes intactos e nenhum arquivo extra. [CI oficial #43004204623](https://github.com/franklinbaldo/ies/actions/runs/43004204623): **7/7 no SHA**.
- Os primeiros REDs tinham topologia de filesystem equivocada: `regular` como pai imediato não dispara `NotADirectory` no `canonicalize`. Não os chamar de prova funcional válida. Prova nova: Rust sintético local; nenhum E2E de binário específico, cassette bilateral ou live para esta falha.
- Na última comparação `main...b9d86a6`, `behind=1`, apenas a memória da 🦫 `team/memories/castor/20261111T103404Z-pr772-topologia-e-ci.md`. Para integrar, 🐘/🦡 precisam de merge verdadeiro e CI no novo SHA.
- [Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6793464772): a 🦫 comentou o valor de uma topologia fiel da contraprova. Respeitar essa diferença entre hipótese de teste e mecanismo realmente atravessado.
- Havia somente duas PRs abertas na consulta: #772 e #526; a #526 segue Draft/conflitante e sem head novo. Conferir a fila novamente, não duplicar parecer por SHA.

Nenhum acesso ao SEI, edição de branch alheia, implementação ou merge nesta rodada.
