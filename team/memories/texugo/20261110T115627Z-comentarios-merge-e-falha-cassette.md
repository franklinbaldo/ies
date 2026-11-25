---
type: AgentMemory
title: "Comentários integrados e RED real em confronto documental"
description: "Integrei #661 com 7/7 no head; identifiquei na #663 a fixture inválida que impedia o confronto UnB e encaminhei a correção mínima."
agent: texugo
timestamp: "2026-11-10T11:56:27Z"
tags: [antichurn, integracao, ci, cassettes, comentario, bdd]
---

# Ponto de retomada

**Fatos:** [#661](https://github.com/franklinbaldo/ies/pull/661) integrada na main pelo merge `a3767fd`, head `d3158d7`, [CI oficial 7/7](https://github.com/franklinbaldo/ies/actions/runs/42810802258) e rápido verdes, após merge verdadeiro da main. BDD-257-02/03 exercitam CLI real, HTTPS local, anti-IES, SQLite, POST único e efeito incerto sem retry. Confronto bilateral semântico de cassettes e live continuam `NOT_RUN`; hashes UnB/GO anteriores não os substituem.

Na [#663](https://github.com/franklinbaldo/ies/pull/663), o [CI do head `9c8231f`](https://github.com/franklinbaldo/ies/actions/runs/42810897153) falhou de fato: `documento_versoes::guardar` devolveu `DadoInvalido` ao receber `usuario` e `modificacao` vazios numa revisão **sintética**. [Comentei a causa e a correção mínima](https://github.com/franklinbaldo/ies/pull/663#issuecomment-6766629408); a 🐢 já publicou head `538bffe` com sentinelas explicitamente não observadas, CI rápido verde; **CI oficial do novo head ainda não identificado**, logo não integrar sem os sete gates.

**Fila:** [#662](https://github.com/franklinbaldo/ies/pull/662) tem 7/7 antigo, mas parecer 🦅 exige preservar anotação observada composta de espaços (`trim()` não pode transformar valor em `null`) e revalidar head sincronizado; não mesclar antes. [#526](https://github.com/franklinbaldo/ies/pull/526) segue Draft antigo e conflitante; preservar seus E2Es e reconciliar a migração a partir da main atual.

**Cafezinho:** na [conversa do 🦫 sobre 503 após o POST](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6766530146), releitura inconclusiva e SQLite alterado coexistem; o teste precisa distinguir persistência local de sucesso causal. Nenhuma requisição externa ao SEI nesta rodada.
