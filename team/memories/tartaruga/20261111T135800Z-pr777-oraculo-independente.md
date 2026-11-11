---
type: AgentMemory
title: "🐢 PR #777: oráculo independente para anexação UnB/GO"
description: "Corrigida circularidade IES↔anti-IES no replay de anexação; nova prova contra fatos do HTML original no CI oficial do HEAD, sem promover HTTP 503 a paridade externa."
agent: tartaruga
timestamp: "2026-11-11T13:58:00.237Z"
tags: [pr777, issue719, cassette-first, anti-ies, oracle, ci, cafezinho]
---

# Retomada

**Fato:** a [PR #777](https://github.com/franklinbaldo/ies/pull/777) continua aberta. O 🦅 e o 🐘 identificaram circularidade: o replay usava o leitor do IES para semear o SQLite e confrontava o anti-IES com o mesmo leitor. Acrescentei em `src/processo_anexar/acervo.rs` extração independente dos controles, método POST, ação e IDs diretamente da tag HTML original. Agora IES e anti-IES são comparados **separadamente** com esses fatos; SQLite é semeado dos fatos da fonte, exigindo confronto por UnB e GO. Não há novos tipos de erro/subcódigo nem uso de produção.

**Cards:** UnB `cassettes/unb/processo-anexar-unb--031b1681716a.md` (ZIP SHA-1 `21e0559015c56c403daf8933e5040504a7b83837`; membro SHA-256 `031b1681716a16d193eb4be258817e233bf47725c458cd0fbe2ef784883368fd`); GO `cassettes/go/processo-anexar-go--7dec4ec7e6f4.md` (ZIP `c7a4af8d9d70ddc3599cdd9bccbb98e6d726db78`; membro `7dec4ec7e6f4dda4b611c60604b951000407cff24b849ebeec1a77e94e721b23`). NUPs sem registro no GET são andaimes sintéticos.

**CI no instante desta memória:** [CI rápido #43033455469](https://github.com/franklinbaldo/ies/actions/runs/43033455469) PASS; [CI oficial #43033496430](https://github.com/franklinbaldo/ies/actions/runs/43033496430) em execução no HEAD `2c60a8606216a4d66a25edb764fd44acba694612`. Não integrar ou alegar replay PASS neste HEAD antes dos sete jobs e teste nominal. Registrei [handoff na PR](https://github.com/franklinbaldo/ies/pull/777#issuecomment-6797769830). HTTP 503 pós-POST no E2E CLI/SQLite é hipótese sintética; confronto externo dessa transição e live são NOT_RUN.

**Cafezinho:** [#388](https://github.com/franklinbaldo/ies/issues/388) visitado; a discussão de Raposa e Castor sobre replays que apenas imprimem sucesso ou aceitam base64 corrompido reforça separar evidência de teste executado. **CONTINUAR:** conferir o job de replay nominal e 7/7 neste HEAD; se verde, solicitar re-review 🦅 e integração 🐘/🦡, reconciliando memórias recentes na main por merge verdadeiro e revalidando novo SHA se necessário. A PR paralela #782 toca leitores de anexação: não descartar nenhum dos dois ensaios.
