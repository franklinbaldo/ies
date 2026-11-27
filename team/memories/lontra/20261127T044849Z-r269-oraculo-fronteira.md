---
type: AgentMemory
title: "Elegibilidade ZIP: oráculo e fronteira da árvore"
description: "Distinguir número declarado, src cru e barra; reaproveitar árvore final já lida."
agent: lontra
timestamp: "2026-11-27T04:48:49Z"
tags: [exportacao, cassette, elegibilidade]
---

# Retomada

[PR #899](https://github.com/franklinbaldo/ies/pull/899#issuecomment-7210138553) DRAFT, HEAD 74abbf6d, 3/17 sobre main 59f51068. #885 integrada; [R278](https://github.com/franklinbaldo/cafezinho/issues/21#issuecomment-7209565061) mantém #886 antes da janela RED. Todos exige a15 declarado não vazio; src ausente/duplicado vazio e a3 com espaços divergem da hipótese literal estrita (modelo 14/14, não wire). ArvoreAberta retém a última árvore conferida; o fluxo descarta HTML antes da exportação. Após RED, preferir projeção interna mínima nessa fronteira, sem GET ou URL assinada na saída. Composição do ZIP é lacuna separada já prevista na #889. Rust/replay/E2E/live e prek local NOT_RUN. Sem alteração de branch, dispatch ou merge.
