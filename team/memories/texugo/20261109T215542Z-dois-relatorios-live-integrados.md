---
type: AgentMemory
title: "Dois relatórios live integrados; causalidade não é estado"
description: "Mesclei #606 e #605 com CI 7/7 e rápido verdes nos heads, mantendo distintas confirmação de PDF e atribuição inconclusiva; #602 aguarda sincronização."
agent: texugo
timestamp: "2026-11-09T21:55:42.744Z"
tags: [antichurn, integracao, ci, live, causalidade, cassettes]
---

# Retomada

- [#606](https://github.com/franklinbaldo/ies/pull/606) integrada por merge `1361ba5`, no head `b6d632c` com CI oficial **7/7** e CI rápido verde, `behind=0` antes do merge. Relata segunda inclusão de PDF externo autorizada na UnB de treinamento: prévia sem upload, confirmação única **saída 0**, documento `0012273`, SHA-256 conferido em releitura. Preserva registro histórico da primeira tentativa, que incluiu `0012272` mas saiu 3 indevidamente. `--confirmar` público continua fechado; não houve novo POST nesta rodada.
- [#605](https://github.com/franklinbaldo/ies/pull/605) integrada por merge `302a81a` no head `ef720cd`, CI oficial **7/7** e rápido verde no head; integração direta GitHub teve êxito embora a branch estivesse atrasada apenas em trabalho paralelo documental/memórias. Relata prévias de atribuição PASS em UnB, mas confirmação única **inconclusiva** (saída 3); releitura mostra usuário pretendido, **não prova causalidade do POST**. Resíduo possível, sem retry ou remoção. Ambos são relatórios, não novos E2Es sintéticos nem confrontos de cassette. **Não executei CI pós-merge**.
- [#602](https://github.com/franklinbaldo/ies/pull/602) continua Ready, v27 de andamento já reconciliada com v26 de blocos, **PASS sintético** e integridade UnB/GO inspecionada; precisa merge verdadeiro da main pelo 🦫 (comparação já indicava 29 behind) e novo CI 7/7. Não reiniciar E2E nem inventar gate de cassette/live. [#607](https://github.com/franklinbaldo/ies/pull/607) iniciou RED de histórico de documento cancelado, CI em execução; não integrar só porque CI rápido estava verde. [#526](https://github.com/franklinbaldo/ies/pull/526) segue Draft antiga e conflitante; próxima migração deve ser recalculada na main real quando retomar.
- No [Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6751983963), a 🦊 já destacou a diferença entre estado visto e autoria do efeito; não há necessidade de repetir o tema como regra ou novo gate.

Nenhuma escrita em produção, novo POST externo, credencial, subcódigo, rebase/force-push ou edição de branch alheia nesta rodada.
