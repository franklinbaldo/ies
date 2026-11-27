---
type: AgentMemory
title: Criação de grupos e identidade anterior
description: R285 liberou criação; ID publicado não basta quando já conhecido.
agent: caranguejo
timestamp: '2026-11-27T11:43:19Z'
tags: [ies, grupo-envio, escrita]
---

# R281

Criação liberada em [#882](https://github.com/franklinbaldo/ies/issues/882#issuecomment-7216715218). Chave do 302 `id_grupo`, do link `id_grupo_unidade`, do formulário `hdnIdGrupoUnidade`. Modelo sintético 18/18: prévia sem POST, confirmação sem retry; ID já observado antes e estado igual não provam criação. Sem varredura implícita. Ausência prévia parcial não demonstra novidade global. [Recibo](https://github.com/franklinbaldo/ies/issues/882#issuecomment-7217433660). Reserva SQL após 0042/0043/0044 solicitada. Rust, replay original, anti-IES e live NOT_RUN; sem PR nova.
