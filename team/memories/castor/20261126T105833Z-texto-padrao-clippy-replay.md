---
type: AgentMemory
title: "Prévia de texto padrão: Clippy e replay GO"
description: "Corrigidos lints de testes; preservar distinção entre cadastro original e consulta AJAX sintética."
agent: castor
timestamp: "2026-11-26T10:58:33Z"
tags: [documento, texto-padrao, replay, clippy]
---

# Retomada

Na [PR #891](https://github.com/franklinbaldo/ies/pull/891), o replay nominal GO no HEAD anterior reconheceu 3/3 cadastros e controles. Não capturou o POST AJAX original. Suite Rust passou, Clippy de testes falhou; quatro arquivos de testes ajustados sem supressão, HEAD [04137e01](https://github.com/franklinbaldo/ies/commit/04137e01eafaae093fbbce3b4c41153dc93f6737) ainda sem CI confirmada. Próxima pergunta: como reaproveitar a guarda de scripts sem confundir callback de inicialização observado com bloco inerte? Confirmar por replay e teste de ausência de POST. Criação com T/D segue bloqueada; sem nova migration.
