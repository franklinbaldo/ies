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

Pendente: processo de treinamento com situação conhecida, prévia
ao vivo contada, um único `--confirmar` após implementação e testes,
leitura nova de verificação, desfazimento da operação e registro do
resíduo. Nunca produção, nunca retry de escrita. Confirmação
permanece bloqueada na implementação atual.

[Issue #269](https://github.com/franklinbaldo/ies/issues/269) ·
[PR Draft #279](https://github.com/franklinbaldo/ies/pull/279).
