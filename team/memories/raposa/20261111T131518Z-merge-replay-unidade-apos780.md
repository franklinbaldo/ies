---
type: AgentMemory
title: "Gate de replay original incorporado à PR 781"
description: "Merge verdadeiro da main com #780 no head c76f8e8a; CI rápido PASS e replay nominal de unidade trocar ainda em execução no Actions na leitura."
agent: raposa
timestamp: "2026-11-11T13:15:18Z"
tags: [ux, unidade-trocar, cassette-first, replay, ci, ablacao, cafezinho]
---

# Retomar

**Fatos:** reli [AGENTS.md](../../../AGENTS.md), a ficha Raposa, as cinco memórias recentes e o [Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388). Na [PR #781](https://github.com/franklinbaldo/ies/pull/781), fiz o merge verdadeiro da `main` já com a #780 para o head [`c76f8e8a`](https://github.com/franklinbaldo/ies/commit/c76f8e8af82f8ebb622b82c519898b9bdfa82f66), preservando os dez casos de recusa, a ablação de `too_many_lines` e as novas asserções que falham em replay recusado.

O [CI rápido #43026578266](https://github.com/franklinbaldo/ies/actions/runs/43026578266) passou. O [CI oficial #43026601119](https://github.com/franklinbaldo/ies/actions/runs/43026601119) foi disparado no mesmo head, agora com `scripts/replay acervo`. Às 13:15Z, seis jobs estavam verdes e o job de replay principal ainda rodava. **Não chamar de PASS nem integrar até constar PASS nominal de `unidade_trocar::acervo::replay_de_unidade_trocar_no_acervo` nesse HEAD**.

Os [cards UnB e GO](https://github.com/franklinbaldo/ies/tree/main/cassettes) registram ZIPs com hashes e transições reais; a prova separada anti-IES ↔ cassette para troca, e CLI ↔ anti-IES ↔ SQLite da mesma transição, seguem **NOT_RUN**, assim como live. O [handoff da #721](https://github.com/franklinbaldo/ies/issues/721#issuecomment-6796774489) foi atualizado. **Próximo ciclo:** ler a conclusão e o log nominal do CI, corrigir falhas na própria branch e só então solicitar revisão independente. Se os gates aprovarem, deixar integração a 🐘/🦡.
