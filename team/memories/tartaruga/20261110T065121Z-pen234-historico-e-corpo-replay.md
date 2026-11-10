---
type: AgentMemory
title: "PEN2/3/4: replay bilateral de histórico e corpo sem inventar versões"
description: "PR #645 acrescenta três confrontos independentes com cassettes PEN2–PEN4; inspeção SHA válida, CI 7/7 no head sincronizado; sem novo E2E CLI nem live."
agent: tartaruga
timestamp: "2026-11-10T06:51:21Z"
tags: [bdd60, anti-ies, cassette-first, pen2, pen3, pen4, navegacao, sqlite]
---

# Para retomar

**Fato:** [PR #645](https://github.com/franklinbaldo/ies/pull/645) está Ready para revisão/integradores. [Inspeção #42770158262](https://github.com/franklinbaldo/ies/actions/runs/42770158262): três cassettes de PEN2, PEN3, PEN4, 90 interações, hashes SHA-1/SHA-256 conferidos. O manifesto temporário saiu da PR. Os três replays novos passam pelo leitor real IES das respostas originais, montam SQLite efêmero com histórico e apenas o corpo da revisão capturada, e confrontam a resposta do anti-IES HTTP. Recusas de outro processo (404), POST (405), revisões persistidas e proveniência do corpo também são verificadas.

**Validação:** [CI oficial #42770812680](https://github.com/franklinbaldo/ies/actions/runs/42770812680) **7/7 PASS** e [rápido #42770791202](https://github.com/franklinbaldo/ies/actions/runs/42770791202) PASS no SHA `4022de0f`, depois de merge verdadeiro da `main`. Nenhum RED funcional nem mudança produtiva: evidência independente adicional, não correção do servidor. CLI binário ↔ anti-IES ↔ SQLite: somente **PASS sintético preexistente** reexecutado; cassette PEN2/3/4: **PASS parcial novo**; treinamento/live/REA: **NOT_RUN**. Corpos históricos ausentes continuam desconhecidos.

**Coordenação:** [handoff #60](https://github.com/franklinbaldo/ies/issues/60#issuecomment-6761055941), [Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6761059235). A memória publicada diretamente na `main` pode deixar a PR um commit atrás; conciliar por merge verdadeiro e revalidar gates se o head mudar. Não fechar a issue #60, que continua parcial. Nenhum SEI produtivo, segredo, novo subcódigo ou TLS reduzido.
