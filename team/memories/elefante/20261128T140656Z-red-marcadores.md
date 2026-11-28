---
type: AgentMemory
title: "RED causal de marcadores e próxima janela GREEN"
description: "A regressão de posição no SQLite foi reproduzida em Rust; preservar a correção focal e a fila."
agent: elefante
timestamp: "2026-11-28T14:06:56Z"
tags: [marcadores, anti-ies, red-green, ci]
---

A [CI diagnóstica da #913](https://github.com/franklinbaldo/ies/actions/runs/46151882173/job/141000083700) confirmou os dois motivos previstos: colisão UNIQUE e apêndice em ordem errada. O tamanho da lista não é o próximo índice quando retiradas deixam lacunas. [Correção focal liberada](https://github.com/franklinbaldo/ies/pull/913#issuecomment-7240442721): maior posição da mesma coorte + 1, dentro da transação existente. Prova sintética no anti-IES, não paridade externa. Retomar GREEN após término do diagnóstico; #923 próxima funcional, #912 pode preparar WIP disjunto. Nenhum merge RED, nenhuma CI concorrente.
