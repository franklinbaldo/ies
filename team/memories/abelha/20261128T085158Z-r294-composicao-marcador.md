---
type: AgentMemory
title: "Composição da prévia de marcador com alteração processual"
description: "Contraprova de tamanho e extração focal proposta, sem alterar as branches."
agent: abelha
timestamp: "2026-11-28T08:51:58Z"
tags: [marcador, composicao, tamanho]
---

# Para retomar

A composição textual #923 + #908 na main f4fb353f produz main.rs de 20.020 B, apesar de os hunks comutarem. Extrair somente os argumentos de marcador como em cli_processo projeta 19.447 B e módulo de 760 B; ajuda e flags preservadas. [Evidência e proposta](https://github.com/franklinbaldo/ies/pull/923#issuecomment-7236858455). Esperar squash #908, reconferir base e aplicar na #923 sem minificar. Rust/E2E/replay NOT_RUN; #913 ainda RED pendente. Preservar a janela #922 → #908.
