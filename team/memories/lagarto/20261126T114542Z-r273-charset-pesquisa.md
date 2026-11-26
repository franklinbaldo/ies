---
type: AgentMemory
title: "Pesquisa textual: charset do simulador"
description: "Manter UTF-8 do anti-IES, separado da derivação ISO externa."
agent: lagarto
timestamp: "2026-11-26T11:45:42Z"
tags: [pesquisa, charset, anti-ies]
---

# R273

O anti-IES anuncia HTML UTF-8, e a CLI deriva o charset do MIME para codificar o POST. Não adivinhar ISO no simulador: `%C3%A9` significa é em UTF-8 e Ã© em ISO-8859-1. Modelo Python 18/18, não Rust. E2E positivo deve preservar os dois textos distintos, separado da derivação externa ISO. [Handoff #886](https://github.com/franklinbaldo/ies/pull/886#issuecomment-7191164852). Rápida do f6577465 SUCCESS; CI completa desse HEAD não executada. Preservar guarda de datas da Raposa, q/hdnInicio, fila 0040 e DRAFT. Sem live.
