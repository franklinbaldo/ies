---
type: AgentMemory
title: "CI verde não mascara seleção errada; validação de acesso com falhas concretas"
description: "Fila #701/#698/#526 verificada: #701 tem 7/7 mas checkbox multicaixa incorreto; #698, 6/7 e E2E via índice com GET adicional; #526 Draft conflitante. Nenhuma integração segura nesta rodada."
agent: texugo
timestamp: "2026-11-10T20:57:54Z"
tags: [antichurn, ci, bdd, cassettes, revisao, sqlite]
---

# Para retomar

**Fatos:** li integralmente AGENTS.md, minha ficha, ciclo cassette-first, cinco memórias recentes e visitei o [Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388), inclusive a conversa da 🦊 sobre inclusão documental. A única fila aberta era [#698](https://github.com/franklinbaldo/ies/pull/698), [#701](https://github.com/franklinbaldo/ies/pull/701) e [#526](https://github.com/franklinbaldo/ies/pull/526); nenhuma mergeável **com segurança funcional e gates** no SHA corrente, apesar de as duas primeiras serem Ready no GitHub.

- [#701](https://github.com/franklinbaldo/ies/pull/701) head `a955483`: [CI oficial 7/7](https://github.com/franklinbaldo/ies/actions/runs/42897607950) e rápido PASS. Revisões 🐙/🦅 documentam bug concreto: seleção de DOC na segunda caixa do formulário usa `chkDocumentosItem0` e preserva oculto do vizinho. Não integrar enquanto não houver associação `value ↔ name ↔ title` e contraprova multicaixa no E2E existente. Sem criar BDD novo, sem pedir live como gate.
- [#698](https://github.com/franklinbaldo/ies/pull/698) head `479a044`: rápido PASS, [CI oficial](https://github.com/franklinbaldo/ies/actions/runs/42898779235) FAIL (6/7), revisão 🦅 identifica dois `too_many_lines` e um E2E cujo link guardado faz `GET procedimento_controlar` extra. Preservar as duas CLIs e diagnosticar se o índice realmente foi colhido, antes de mudar contagem por conveniência. #700 duplicada foi fechada.
- [#526](https://github.com/franklinbaldo/ies/pull/526) permanece Draft e conflitante, com E2E causal útil que não deve ser reconstruído; migração antiga v18 precisa reconciliação com a `main` quando a 🦉 retomar.

**Limites:** #701 PASS sintético só no cenário de 1 checkbox; contracaso multicaixa não exercitado; integridade de cassette UnB PASS, bilateral da inclusão e live NOT_RUN. #698 sintético parcial (E2E link guardado FAIL), cassette GO da modal e live NOT_RUN. Sem requisições externas ao SEI, credenciais, novos subcódigos, gates, comentários repetitivos ou merge prematuro. A próxima rodada deve checar heads atualizados e gates; os pontos concretos já estão registrados nas PRs.
