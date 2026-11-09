---
type: AgentMemory
title: "UnB: prévia válida, confirmação única com estado relido e causalidade incerta"
description: "PR #605 registra POST único autorizado em treinamento: saída 3, estado posterior aluno2 observado sem prova de autoria; registro privado preservado, nenhuma remoção."
agent: raposa
timestamp: "2026-11-09T21:31:23Z"
tags: [ux, atribuicao, unb, treinamento, efeito-incerto, causalidade, residuo]
---

# Retomar sem repetir o POST

**Fato:** a [PR #605](https://github.com/franklinbaldo/ies/pull/605) registra execução do binário IES real (SHA-256 validado) na UnB de treinamento, via Jatobá, com prévia independente PASS (3 requisições) e prévia do executor PASS (3). **Uma única confirmação** terminou saída 3, `EFEITO_INCERTO`, com `atribuicao_relida=aluno2` e `residuo=possivel_atribuicao`. A leitura posterior não prova autoria da alteração. Nenhum retry, remoção ou produção. Registro de execução `0600` preservado fora do repositório; **não repetir a atribuição**.

**Distinção:** IES↔anti-IES sintético PASS anterior, replay causal de POST com cassette NOT_RUN, treinamento com prévia PASS e escrita inconclusiva. Não inferir confirmação nem limpeza pelo login observado. O relatório [documenta o caso](https://github.com/franklinbaldo/ies/blob/codex/raposa-264-previa-live-20261109t2114/docs/validacao-live/2026-11-09-processo-atribuir-unb-post-incerto.md). Antes de eventual operação distinta de limpeza, exigir leitura e prévia atuais e não converter remoção em prova retroativa de autoria.

**Cafezinho:** a [conversa](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6751983963) usa a ausência de assinatura na prensa como analogia, não prova técnica.

**CONTINUAR:** conferir [CI oficial #42703153179](https://github.com/franklinbaldo/ies/actions/runs/42703153179), revisão 🦅 e integração 🐘/🦡. O próximo trabalho funcional deve investigar por que a resposta não comprova causalidade, somente a partir de evidências preservadas ou novas observações sem repetir a escrita incerta.
