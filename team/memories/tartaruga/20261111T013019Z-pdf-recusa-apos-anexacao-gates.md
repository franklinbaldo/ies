---
type: AgentMemory
title: "PR #734: recusa documental entre processos após anexação validada em CLI real"
description: "Merge verdadeiro da main, CI oficial 7/7 no SHA a389fff93; teste negativo CLI/TLS/anti-IES/SQLite PASS sintético; cassettes/live sem confronto da variante."
agent: tartaruga
timestamp: "2026-11-11T01:30:19.817Z"
tags: [anti-ies, issue496, bdd60, pdf, recusa-tipada, cli-real, sqlite, cassette-first, ci]
---

# Retomada

**FEITO:** [PR #734](https://github.com/franklinbaldo/ies/pull/734), issue [#496](https://github.com/franklinbaldo/ies/issues/496), BDD-60-05 já especificado na [#60](https://github.com/franklinbaldo/ies/issues/60): o documento PDF de A não se torna pertencente ao processo B só porque B foi anexado a A. A CLI real recusa `ies documento 050000001 --processo B --saida DIR --sem-cache`, com e sem `--ocultar-inacessiveis`, usando causa original `NumeroSeiAusenteDaArvoreDoProcesso`, exit 1 e sem criar arquivo; o conteúdo autorizado de A é idêntico antes/depois. Proxy TLS loopback confirma somente 2 GETs de download e 1 POST de anexação, sem retry, com mesmo SQLite efêmero. Apenas testes modificados.

**Sincronização/GREEN:** branch `tartaruga/496-pdf-recusa-apos-anexacao-20261110` incorporou `main` por merge verdadeiro de dois pais [`a389fff93`](https://github.com/franklinbaldo/ies/commit/a389fff93d2459275db3462b2ff40c15d9e770b3), sem sobreposição ou force-push. [CI completo #42938647577](https://github.com/franklinbaldo/ies/actions/runs/42938647577) **7/7 PASS** e [rápido #42938538868](https://github.com/franklinbaldo/ies/actions/runs/42938538868) **PASS**, ambos no SHA do merge, `behind=0` na conferência. PR Ready, mergeable e aberta na última consulta. CI anterior #42937197455 falhou no teste não relacionado de comentários com `AlreadyExists` em diretório TLS efêmero; não confundir isso com negativa do PDF.

**Provas:** nível 1 IES↔anti-IES/SQLite **PASS sintético** no head; nível 2 específico **NOT_RUN**: cards UnB/GO `processo-anexar` 102 HTTP e UnB `documento-numero-e-busca` 15 HTTP com hashes inspecionados [#42924729149](https://github.com/franklinbaldo/ies/actions/runs/42924729149) / [#42920191914](https://github.com/franklinbaldo/ies/actions/runs/42920191914), mas sem captura do negativo após anexação; nível 3 live/REA **NOT_RUN**. RED funcional novo não demonstrado. Sem produção, credenciais, novos subcódigos ou relaxamento TLS.

**CONTINUAR:** 🦅 revisar e 🐘/🦡 integrar #734. Depois retomar [PR #724](https://github.com/franklinbaldo/ies/pull/724), ainda aberta e bloqueada em CI [#42935178443](https://github.com/franklinbaldo/ies/actions/runs/42935178443): Clippy `estado()` 101/100 linhas e teste HTTP aguardando 302 que a rota não devolveu; a branch está defasada e exige merge verdadeiro/correção. A prioridade nova de antipatterns em AGENTS.md deve orientar novas fatias, não invalidar conclusões desta. [Handoff #496](https://github.com/franklinbaldo/ies/issues/496#issuecomment-6784283661); [Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6784291240) visitado e comentado. Issues #496/#60 continuam abertas.
