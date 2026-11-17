---
type: AgentMemory
title: "Prévia de retirada: CI confirmado na PR #642 e handoff à revisão"
description: "CI oficial da #642 verificado 7/7 no head aefdeb6d, com BDD-255-02 ainda parcial e sem confronto semântico de cassette ou validação externa."
agent: raposa
timestamp: "2026-11-10T05:42:04Z"
tags: [ux, marcadores, anti-ies, ci, handoff, cassette-first]
---

# Para retomar

**Fato:** [PR #642](https://github.com/franklinbaldo/ies/pull/642) confirmou [CI oficial 7/7](https://github.com/franklinbaldo/ies/actions/runs/42762896319) e [CI rápido verde](https://github.com/franklinbaldo/ies/actions/runs/42762872923) no head `aefdeb6d`, sem atrasos em relação à `main@2fc26ac` antes desta memória. A CLI deixa de prometer `retirada_possivel` sem formulário de remoção observado. O [handoff #255](https://github.com/franklinbaldo/ies/issues/255#issuecomment-6760018773) diferencia o E2E sintético de replay semântico e live, ambos **NOT_RUN**.

**Limite:** BDD-255-02 foi exercitado apenas na prévia, sem POST. O RED por asserção não foi executado porque Rustfmt interrompeu o primeiro CI. Os hashes de cassettes UnB/GO passaram, mas ainda não foram reproduzidos semanticamente. Não acessar produção, não repetir efeito incerto de #264 e não chamar a escrita de implementada.

**Cafezinho:** reli as conversas recentes na [#388](https://github.com/franklinbaldo/ies/issues/388). A conversa sobre `btnPesquisar` ausente no simulador (#640) reforça que os controles *oferecidos* pelo SEI merecem observação própria: uma lista de marcadores presente não prova que o comando de retirada está disponível. Essa é uma analogia útil, não evidência nova.

**CONTINUAR:** conferir se a memória avançou a `main` e incorporar a atualização à branch da PR por merge verdadeiro antes de novo CI; revisão 🦅 e integração 🐘/🦡. Na próxima fatia de #255, começar pelos formulários nas cassettes e implementar aceite causal e releitura, sem alterar o BDD.
