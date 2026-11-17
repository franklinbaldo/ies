---
type: AgentMemory
title: "BDD-60-05: PEN1 liga corpo observado à versão do histórico"
description: "PR #643 confronta árvore, histórico e corpo PEN1 com anti-IES HTTP/SQLite; somente versão capturada recebe conteúdo; CI oficial 7/7."
agent: tartaruga
timestamp: "2026-11-10T05:54:41.020Z"
tags: [bdd60, pen1, cassette-first, anti-ies, navegacao, sqlite, proveniencia]
---

# Para retomar

**Fato:** a [PR #643](https://github.com/franklinbaldo/ies/pull/643) acrescenta replay da cassette original PEN1 de liberações/versões. A [inspeção #42763265587](https://github.com/franklinbaldo/ies/actions/runs/42763265587) validou SHA-1/SHA-256 e 27 interações, incluindo GET de histórico e de corpo. A identidade DOC/NUP/id vem da árvore, autores/unidades/datas da lista e texto/tipo/versão do GET de corpo da mesma identidade; as revisões não observadas usam somente `<p></p>` estrutural. A projeção SQLite → anti-IES HTTP passa pelos leitores reais do IES; GET de outro processo recusa 404, POST 405, revisões persistidas inalteradas. GO/UFRJ permanecem cobertos apenas pelo histórico. O ZIP é escolhido pelo card, não hardcoded.

**Validação:** [CI completo #42764676330](https://github.com/franklinbaldo/ies/actions/runs/42764676330) **7/7 success** e [rápido #42764669957](https://github.com/franklinbaldo/ies/actions/runs/42764669957) **PASS** no head `41e662c3`. Falhas intermediárias E0382 e Clippy corrigidas sem supressões; não houve RED funcional nem mudança produtiva. **Provas:** CLI ↔ anti-IES ↔ SQLite PASS sintético apenas nas regressões preexistentes; confronto independente PEN1 PASS parcial novo; treinamento/live/REA NOT_RUN.

**Coordenação:** [handoff issue #60](https://github.com/franklinbaldo/ies/issues/60#issuecomment-6760199246), [Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6760005791) visitado e comentado. Revisão 🦅 e integração 🐘/🦡 pendentes; a gravação de memória na `main` pode adiantar essa branch um commit e exigir merge verdadeiro/revalidação. Não enviar ao SEI, nem produzir novos subcódigos ou reduzir TLS.
