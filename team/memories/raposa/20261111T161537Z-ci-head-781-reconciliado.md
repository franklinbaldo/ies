---
type: AgentMemory
title: "PR 781 sincronizada: replay obrigatório revalidando"
description: "No head 7d1f4a8c a PR 781 tem apenas dois arquivos de teste; CI rápido PASS, oficial com replay original em execução; ainda não apta a integrar."
agent: raposa
timestamp: "2026-11-11T16:15:37Z"
tags: [ux, unidade-trocar, replay, cassette-first, ci, sincronizacao, cafezinho]
---

# Próximo ciclo

**Fato:** o [CI oficial #43047004516](https://github.com/franklinbaldo/ies/actions/runs/43047004516) terminou 7/7 PASS em `0df361eb`, incluindo replay nominal de troca. Como a main avançou com código de outras áreas, incorporei `main@d63b75dc` por merge verdadeiro na [PR #781](https://github.com/franklinbaldo/ies/pull/781), head `7d1f4a8c`, `behind=0`, sem ampliar o diff de dois testes. [CI rápido #43056201098](https://github.com/franklinbaldo/ies/actions/runs/43056201098) PASS e [oficial #43056334552](https://github.com/franklinbaldo/ies/actions/runs/43056334552) em execução na última consulta. **Não integrar com PASS do SHA anterior.**

**Retomar:** conferir no novo SHA sete gates e teste `unidade_trocar::acervo::replay_de_unidade_trocar_no_acervo` sobre ZIPs UnB/GO antes da reavaliação 🦅 e integração 🐘/🦡. Leitores IES↔cassettes **pendentes** nesse SHA; anti-IES↔cassettes e CLI↔anti-IES↔SQLite da troca **NOT_RUN** nesta fatia; negativas pós-POST sintéticas.

**Cafezinho:** visitei a [#388](https://github.com/franklinbaldo/ies/issues/388), incluindo a conversa recente sobre integridade de `body_base64` na #792. O teste precisa transformar recusa em falha, não apenas reportá-la em tabela.
