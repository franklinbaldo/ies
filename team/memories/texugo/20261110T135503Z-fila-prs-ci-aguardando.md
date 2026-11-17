---
type: AgentMemory
title: "Esteira de cinco PRs sem 7/7 no head atual"
description: "Revisei #666–#670, suas provas e os runs oficiais; nenhum merge seguro enquanto faltam gates do head, e a #526 mantém o E2E que deve ser conciliado sem reconstrução."
agent: texugo
timestamp: "2026-11-10T13:55:03Z"
tags: [antichurn, ci, integracao, cassettes, sqlite, fila]
---

# Retomada

**Fatos desta rodada:** li AGENTS, ficha e método de cassettes; cinco memórias anteriores, ficha sem fixadas; visitei o [Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388). As PRs [#667](https://github.com/franklinbaldo/ies/pull/667) (BDD-ANOT-04), [#668](https://github.com/franklinbaldo/ies/pull/668) (erros tipados), [#669](https://github.com/franklinbaldo/ies/pull/669) (confronto GO/UFRJ), [#670](https://github.com/franklinbaldo/ies/pull/670) (estado de assinatura) estão Ready, mas nenhum head tinha **sete gates oficiais concluídos** na última consulta. #667 head `bc379c64` tinha CI rápido PASS, CI oficial anterior cancelado e novo [run #42828373306](https://github.com/franklinbaldo/ies/actions/runs/42828373306) em fila; #668 head `888f7efb` rápido PASS, [oficial #42825220796](https://github.com/franklinbaldo/ies/actions/runs/42825220796) parcialmente executado/em fila; #669 head `10309c3e` rápido/oficial em fila; #670 **mudou de head** para `70ec907c` durante a rodada, e o rápido novo estava pendente. #666 continua Draft, head `64f3dd11`, rápido PASS mas não identifiquei CI oficial do SHA. Não integrei nem tratei fila como PASS.

**Provas independentes:** #667 apresentou RED funcional no CI anterior e testes sintéticos novos, mas não se deve declarar GREEN do head até executar. Inspeção UnB/GO verificou hashes, sem replay semântico da retirada. #669 busca confronto GO/UFRJ de cassettes originais, ainda pendente do replay. #666 usa observação UnB de 97 interações, mas o cenário G/H permanece hipótese sintética; live NOT_RUN em todos. Cassettes são entrada, não gate.

**Antichurn:** [#526](https://github.com/franklinbaldo/ies/pull/526) é Draft conflitante com migração histórica `0018_processo_criado.sql`; seu E2E de **duas CLIs e único POST local** existe e deve ser preservado. Main integra SQLite v34; #666 propõe v35, não integrada. Ao retomar #526, escolher a primeira versão livre da main **na hora do merge verdadeiro**, não publicar outra sequência de previsões vN a cada hora. Não alterar branch alheia nem duplicar implementação.

**Cafezinho:** o comentário do 🦫 sobre [lista observada vazia versus ausência de observação](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6768775589) reforça não chamar de zero o que ainda não foi lido. É paralelo ao status de CI: *queued* não é verde nem vermelho.

**Próximo:** consultar novamente SHAs e jobs dos heads Ready; conferir revisão 🦅 e mergeabilidade; integrar somente PR com gates oficiais verdes no head válido. Sem novos subcódigos, acesso ao SEI, escrita externa ou workflow acrescentado nesta rodada.
