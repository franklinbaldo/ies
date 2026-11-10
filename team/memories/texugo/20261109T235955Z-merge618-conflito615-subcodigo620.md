---
type: AgentMemory
title: "Merge de replay UnB, conflito de atribuição após #617 e subcódigo novo identificado"
description: "Integrei #618 com CI 7/7 e rápido; corrigi classificação de replay ignorado; #615 conflita após #617 e a issue #620 registra subcódigo textual novo da guarda de origem."
agent: texugo
timestamp: "2026-11-09T23:59:55Z"
tags: [antichurn, integracao, cassette, ci, erros-tipados, sqlite]
---

# Retomar

**Fatos:** [#618](https://github.com/franklinbaldo/ies/pull/618) merge `a245edb` com CI oficial [#42722918613](https://github.com/franklinbaldo/ies/actions/runs/42722918613) 7/7 e rápido verde no head `461cd29`. A fatia elimina metadados sintéticos `SIMULADO` no teste de corpo UnB, usando versão do rodapé e histórico do mesmo `id_documento`. **Não declarar replay PASS:** o teste `cassette_unb_conteudo_documento_compara_leitores_por_http` ainda usa `#[ignore]`; nenhum run específico dele foi comprovado. [Esclareci na PR](https://github.com/franklinbaldo/ies/pull/618#issuecomment-6754806592). Treinamento NOT_RUN. CI pós-merge não executado.

A [#615](https://github.com/franklinbaldo/ies/pull/615) tem 7/7 e rápido verde no head `5bd7385`, mas está `mergeable=false` após a #617 mudar confirmação de atribuição. [Comentei a conciliação concreta](https://github.com/franklinbaldo/ies/pull/615#issuecomment-6754772440): preservar tanto recusa de origem antes da rede quanto texto **não repita a confirmação**; merge verdadeiro de main e CI novo, sem reconstruir o BDD. [#614](https://github.com/franklinbaldo/ies/pull/614) tem só CI rápido/inspeção de cassette, sem 7/7; já há pareceres precisos 🦅/🦫 sobre release, transação, manifesto e sincronização, não repetir comentários. [#619](https://github.com/franklinbaldo/ies/pull/619) segue Draft RED; [#526](https://github.com/franklinbaldo/ies/pull/526) segue Draft conflitante, migração após v28 deve ser recalculada.

**Regressão de política tipada:** o merge #617 introduziu `OrigemForaDoTreinamento::SUBCODIGO = "origem_fora_do_treinamento"` e chamadores `erro_fonte_http` com string, apesar da [ADR 0005](https://github.com/franklinbaldo/ies/blob/main/docs/arquitetura/0005-erros-por-tipo.md). Registrei [issue #620](https://github.com/franklinbaldo/ies/issues/620) para retirar identidade textual paralela **sem remover** a proteção de origem de treinamento; 🐘 deve especificar BDD antes de implementação. Nenhum acesso à produção, POST externo, credencial ou novo subcódigo foi criado por esta rodada.

[Visitei o Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6754580305): a 🐢 relacionou corpo, histórico e metadados observados em vez de enxertar `SIMULADO`. Boa inspiração para distinguir teste escrito de teste efetivamente executado, não evidência live.
