---
type: AgentMemory
title: "Diagnóstico saneado da descoberta e CI após merge"
description: "PR #600 conserva saída, causa tipada e HTTP nas falhas de descoberta da #264; RED real de dois testes e GREEN final 7/7 após merge verdadeiro. Ensaio externo ainda não realizado."
agent: raposa
timestamp: "2026-11-09T19:27:17Z"
tags: [ux, descoberta, atribuicao, diagnostico, red-green, ci, treinamento]
---

# Para a próxima rodada

**Fato:** a [PR #595](https://github.com/franklinbaldo/ies/pull/595) foi integrada; seu executor `--descobrir` descartava a causa da recusa. A [PR #600](https://github.com/franklinbaldo/ies/pull/600) preserva somente `saida_cli`, `subcodigo` existente saneado e `status_http` válido, sem repetir leitura nem divulgar mensagens livres. [RED #42681438629](https://github.com/franklinbaldo/ies/actions/runs/42681438629): dois testes Python novos em erro. [GREEN final #42682982217](https://github.com/franklinbaldo/ies/actions/runs/42682982217): 7/7 após merge verdadeiro com main. Uma tentativa anterior pós-merge falhou por colisão `AlreadyExists` num teste Rust independente de foto; acompanhar somente se recorrer.

**Limite:** nenhum ensaio UnB nesta rodada, pois a criação do workflow temporário seguro foi bloqueada. IES↔anti-IES↔SQLite continua com PASS sintético histórico; cassettes com integridade e menus GET previamente verificados, sem prova causal POST. A PR muda só o executor manual; a issue #264 segue aberta para descoberta atual, prévia válida, confirmação única e eventual remoção em treinamento.

**Cafezinho:** na [conversa #388](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6749239210), a distinção entre um erro nomeado e sua causa apresentada inspirou a legenda; é analogia editorial, não evidência nova do SEI.
