---
type: AgentMemory
title: "Origem fechada também protege a anotação no domínio"
description: "RED oficial revelou que limpar_anotacao no iesei aceitava origem fechada, apesar da recusa HTTP; PR #485 corrige com variante tipada e testa CLI real com destino não oferecido."
agent: coruja
timestamp: "2026-11-08T16:30:25Z"
tags: [processos, anti-ies, sqlite, reconciliacao]
---

# Para a próxima rodada

**Fato:** [PR #485](https://github.com/franklinbaldo/ies/pull/485), vinculada ao [BDD-333-04/05](https://github.com/franklinbaldo/ies/issues/333), demonstrou [RED](https://github.com/franklinbaldo/ies/actions/runs/42499646826) ao limpar uma anotação depois que a unidade de origem havia sido fechada. O domínio `iesei` agora recusa com `Falha::OrigemNaoAberta` e preserva o SQLite. O [CI GREEN](https://github.com/franklinbaldo/ies/actions/runs/42499896808) passou 7/7; a PR aguarda revisão e integração.

**Distinção:** [inspeção de três cassettes](https://github.com/franklinbaldo/ies/actions/runs/42499528477) verificou 99 interações e hashes, **não** confrontou semanticamente a recusa ou o envio parcial. Nenhum live ou REA. Continuar a partir da PR integrada, sem duplicar o BDD E2E existente nem liberar remoção real antes da validação.

**Cafezinho:** a [prova da prensa](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6723766957) é um lembrete literário da diferença entre identificar vestígios e reconstruir causalidade — analogia, não evidência do SEI.
