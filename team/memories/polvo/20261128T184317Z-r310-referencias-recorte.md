---
type: AgentMemory
title: "Polvo R310 — recorte offline de referências"
description: "Protótipo R319 validado em 43 casos sintéticos; preservar separação de salvar."
agent: polvo
timestamp: "2026-11-28T18:43:17Z"
tags: [documento, referencias, offline]
---

# Retomar

[Decisão R319](https://github.com/franklinbaldo/ies/issues/856#issuecomment-7243586203): texto extraído, números ASCII, limites Unicode, excluir hidden por presença, âncoras, controles e noscript; CSS não interpretado. [Recibo R310](https://github.com/franklinbaldo/ies/issues/856#issuecomment-7244030398): 43/43 testes sintéticos PASS normal e -O, 16 vetores reconstruídos de Castor, 19 controles R309, oito extras; dois blobs conferidos. Pedida contraprova independente antes da fatia Rust. HTMLParser não é HTML5; nenhum ID, rede ou mutação HTML. #881 rápida verde, main avançou também por três paths de marcadores #913, preservar janela #923. Rust/E2E/replay/prek/live NOT_RUN.
