---
type: AgentMemory
title: "BDD-264-01: árvore única preserva atribuição e inclusão documental"
description: "PR #521 conciliada com #525: árvore única em navegacao.rs e barra com inclusão por capacidade e atribuição por observação; CI 7/7 no SHA 0dd9cec."
agent: raposa
timestamp: "2026-11-09T02:17:14Z"
tags: [cli, atribuicao, anti-ies, integracao, sqlite]
---

# Para retomar

Fato: o merge verdadeiro [`0dd9cec`](https://github.com/franklinbaldo/ies/commit/0dd9cecb10e9e3e8e94fbde29f5bdf13b866b832) conciliou a [PR #521](https://github.com/franklinbaldo/ies/pull/521) com a extração da [#525](https://github.com/franklinbaldo/ies/pull/525). A árvore e o trabalho ficam em **um único** `processo_envio/navegacao.rs`, acrescentando atribuição observada sem duplicar o módulo antigo. A ação `Incluir Documento` continua condicionada a `pode_incluir_documento`. SQLite v17 de criação e v18 de atribuição coexistem.

Fato: [CI oficial 7/7](https://github.com/franklinbaldo/ies/actions/runs/42549649437) e [CI rápido](https://github.com/franklinbaldo/ies/actions/runs/42549621706) passaram no novo SHA; PR Ready e mergeável na consulta. [Handoff na #264](https://github.com/franklinbaldo/ies/issues/264#issuecomment-6730644059). Confronto semântico independente com corpos UnB/GO e live permanecem **NOT_RUN**; somente integridade de ZIPs foi verificada em rodada anterior. Próximos: revisão/integração 🦅/🐘/🦡 e BDD-264-02/03, nunca recomeçar 264-01.

Do [Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388), conservar a diferença entre evidência de um estado final e evidência de sua transição causal. A folha `02` continua analogia ficcional, não evidência do SEI.
