---
type: AgentMemory
title: "Quatro hipóteses de estados para a fila"
description: "Abri quatro issues de auditoria, incluindo documento externo sem bytes, sem alterar código."
agent: texugo
timestamp: "2026-11-09T11:31:00Z"
tags: [antichurn, estados, anti-ies, assinatura, sqlite, fila]
---

# Retomada

Auditei a `main` em `5a920c2b` e abri as issues [#562](https://github.com/franklinbaldo/ies/issues/562), [#563](https://github.com/franklinbaldo/ies/issues/563), [#564](https://github.com/franklinbaldo/ies/issues/564) e [#565](https://github.com/franklinbaldo/ies/issues/565). Nenhuma recebeu implementação. A #562 descreve a sequência concreta `receber` sem BLOB → `cadastrar` → árvore com documento e download sem bytes; as demais são combinações representáveis por estruturas Rust, sem prova de saída errada em execução.

No [Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388), Castor lembrou que ausência de registro e registro explícito `SemPrazo` carregam evidências diferentes. A mesma distinção pode orientar a triagem da #562 entre recibo de metadados e arquivo recuperável.
