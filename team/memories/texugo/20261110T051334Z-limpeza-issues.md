---
type: AgentMemory
title: "Limpeza de issues: implementação e prova externa"
description: "Encerrei #264 e #269 por BDD sintético completo; pendências preservadas na #583; roadmap com fila atual de 38 issues e 4 PRs."
agent: texugo
timestamp: "2026-11-10T05:13:34Z"
tags: [antichurn, backlog, bdd, validacao]
---

# Para retomar

Limpeza solicitada pelo proprietário: #264 e #269 encerradas conforme decisão prévia da #583, após confirmar merges #547/#615/#627. #583 preserva POST causal não comprovado, live de sobrestamento NOT_RUN e atribuição UnB inconclusiva (saída 3, possível resíduo, não repetir). #640 continua aberta; conclusão sintética não cancela comparação externa. Corrigi #583: #631 está mesclada em 5f0fb9c, relatório das sete instalações lido na main, sem nova execução nesta rodada. Remoção de anotação preexistente continua pendente na #638.

#56 recebeu fila atual (38 issues, 4 PRs) antes dos snapshots históricos; não retomar fila antiga. #609/#581 continuam bugs; #588–#592 permanecem auditorias separadas; #286 é migração global de erros e #641 só uma fatia. #288 ainda contém defaults silenciosos; #289 ainda importa helpers de outra CLI. Nenhuma PR integrada nem código alterado nesta limpeza.

Cafezinho #388 visitado. A conversa sobre dado vazio versus não observado reforça separar encerramento da implementação de prova live; é analogia, não validação externa.
