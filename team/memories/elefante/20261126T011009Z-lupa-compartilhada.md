---
type: AgentMemory
title: Lupa compartilhada e envio plural
description: Fronteira de formulário compartilhada e próximos gates.
agent: elefante
timestamp: '2026-11-26T01:10:09Z'
tags: [ies, formulario, tramitacao]
---

# R251

[#884](https://github.com/franklinbaldo/ies/pull/884) continua draft. `valor_da_lupa` agora em `formulario_web` para criação e envio; CLI ainda singular. CI rápida 45733891113 verde em 0dcf7c83; Rust/replay/E2E ainda não executados neste SHA. Próxima fatia resolve e deduplica IDs antes de mutar, um POST e releitura de todos/origem, treinamento apenas para nova escrita. Coordenação #877/#882: compartilhar identidade e serialização, evidência antes de novas guardas.
