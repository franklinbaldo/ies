---
type: AgentMemory
title: "Polvo R311 — referências Rust em DRAFT"
description: "Implementação local da #856 com testes ainda não executados."
agent: polvo
timestamp: "2026-11-28T19:47:00Z"
tags: [documento, referencias, offline]
---

# Retomar

🐙 R311 / R320 — [#926 DRAFT](https://github.com/franklinbaldo/ies/pull/926), HEAD `6952d44bc1d0f3e7a5e22241ac1a6c623c59c63f`, base 16096953. Entrega real de `ies --json documento referencias --arquivo HTML [--proprio DOC] [--nup NUP]`; DOM central fragmento, sem rede, sessão, IDs, reescrita ou mudança em salvar. Regex 1.13.1 já transitiva, agora dependência direta; fronteiras L/N/M/underscore, caractere anterior examinado localmente. 49 vetores matriciais escritos (41 R310 + oito R320), mais dois controles de tipo/autorreferência e E2E CLI local sintético, não executados. Rust/Clippy/rustfmt/replay/prek NOT_RUN (toolchain ausente). Nove arquivos, maior texto não-lock 19.450 B. Solicito contraprova 🦫 Castor da implementação real e revisão do contrato experimental; preservar #923, sem dispatch, READY, merge ou live.

[Direção R320](https://github.com/franklinbaldo/ies/issues/856#issuecomment-7244343045). #881 DRAFT; preservar fila #923.
