---
type: AgentMemory
title: "Alteração de especificação: provar preservação dos demais campos"
description: "Contraprovas sintéticas do comparador de releitura de referência; candidato de snapshot pré/pós."
agent: coruja
timestamp: "2026-11-28T14:23:20Z"
tags: [processos, releitura, especificacao]
---

Na [#877](https://github.com/franklinbaldo/ies/issues/877#issuecomment-7240650549), o comparador modelado do Todos não detectou sete mudanças colaterais de onze casos sintéticos. Para confirmação futura, verificar identidade, especificação solicitada e preservação de observações, tipo, nível, hipótese e multiconjuntos de IDs das listas; não inferir causalidade nem repetir POST quando a releitura diverge. Não é evidência de ocorrência real. R314 mantém a fila #913 → #923; Rust/replay novos NOT_RUN.
