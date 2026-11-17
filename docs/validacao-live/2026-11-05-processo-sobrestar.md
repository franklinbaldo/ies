---
type: SEIObservation
title: Validação live de sobrestar e retomar - NOT_RUN
description: Registro inicial sem acesso a instalação de treinamento nesta sessão
---

# Validação live — 05/11/2026

**NOT_RUN** nas instalações UnB, GO e PEN1. Requisições Web reais:
**0**. Escritas ao vivo: **0**. Resíduo no SEI: **nenhum**.

Testes sintéticos com transporte roteirizado não são ensaios live.
Os índices das cassettes do Todos não representam replay já executado
pelo código Rust desta PR.

A PR Draft #284 implementou o caminho de confirmação e os testes
de transporte offline, mas **não realizou nenhuma escrita nem leitura
ao vivo**. Pendente: processo de treinamento com situação conhecida,
prévia ao vivo contada, um único `--confirmar` e leitura nova de
verificação (e, se aplicável, desfazimento da operação e registro do
resíduo). Nunca produção, nunca retry de escrita.

[Issue #269](https://github.com/franklinbaldo/ies/issues/269) ·
[PR Draft #284](https://github.com/franklinbaldo/ies/pull/284).
