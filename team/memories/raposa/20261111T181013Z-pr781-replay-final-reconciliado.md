---
type: AgentMemory
title: "PR 781 reconciliada; replay assertivo no CI do HEAD final"
description: "CI anterior 7/7 com replay real; merge verdadeiro com main em 9f80d867, CI rápido verde, oficial acionado e ainda pendente."
agent: raposa
timestamp: "2026-11-11T18:10:13Z"
tags: [ux, unidade-trocar, cassette-first, replay, ci, sincronizacao, cafezinho]
---

# Retomar na próxima rodada

**Fatos:** li `AGENTS.md`, a ficha da Raposa, cinco memórias recentes e o [Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388). Conferi o [CI oficial #43065590260](https://github.com/franklinbaldo/ies/actions/runs/43065590260): **7/7 PASS** no SHA anterior `ec59698e`. O log comprova `RUSTUP_TOOLCHAIN=1.99.1 scripts/replay acervo`, 495 cards e 29 ZIPs verificados por hash, 32 cassettes de unidade trocar lidas e teste nominal aprovado; não é confronto do anti-IES para essa transição.

Sincronizei a [PR #781](https://github.com/franklinbaldo/ies/pull/781) com `main@d887c806`, criando merge verdadeiro de dois pais no [HEAD `9f80d867`](https://github.com/franklinbaldo/ies/commit/9f80d867547802263109956504b68abbddcad75e). `behind=0` na conferência, diff só `src/unidade_trocar/{acervo,tests}.rs`. [CI rápido #43075576265](https://github.com/franklinbaldo/ies/actions/runs/43075576265) **PASS**; [CI oficial #43075678387](https://github.com/franklinbaldo/ies/actions/runs/43075678387) iniciado por Draft→Ready, **ainda em execução** na consulta. Atualizei PR e [handoff #721](https://github.com/franklinbaldo/ies/issues/721#issuecomment-6803708816).

**Evidências HEAD atual:** IES↔original **pendente do replay**, anti-IES↔original **NOT_RUN**, CLI real↔anti-IES↔SQLite específico da troca **NOT_RUN**, live **NOT_RUN**. Os dez casos negativos depois do POST são sintéticos. Cards UnB/GO e hashes permanecem documentados na PR.

**CONTINUAR:** ler o job `replay unidade.retornos` no run atual, exigir 7/7 + teste nominal no mesmo SHA, colher nova revisão 🦅, integração por 🐘/🦡. A memória na main pode deixar branch um commit atrás apenas documental, sem exigir novo CI por isso.

**Cafezinho:** discussão do 🦫 sobre dados `body_base64` corrompidos reforça que integridade estrutural e resultados de leitor têm papéis diferentes; não declarar confronto da transição inteira sem prova independente.
