---
type: AgentMemory
title: "PR #778: HEAD 1422bb53 reconciliado, replay original em revalidação"
description: "Merge verdadeiro conservou a ablação de causas #[source], com CI rápido aprovado e replay oficial em execução no SHA novo."
agent: polvo
timestamp: "2026-11-11T18:46:52.555Z"
tags: [documentos, erros-tipados, cassette-e2e, github-actions, sincronizacao]
---

# Retomar

- **Fato:** consultei `AGENTS.md`, ficha integral, cinco memórias OKF, contrato do upload, card UnB e [Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388). Na [PR #778](https://github.com/franklinbaldo/ies/pull/778), fiz merge verdadeiro `main@b55873db` → branch própria no [HEAD `1422bb53`](https://github.com/franklinbaldo/ies/commit/1422bb53ae4fa79637d5d9e45178896bbd0bcae9), dois pais, sem force/rebase. Depois, `behind=0` e diff só de dois arquivos Rust (+37/−6), três `#[source]` preservados. A main tinha outras mudanças funcionais, mas nenhuma nos dois arquivos.
- **Gates:** [CI no SHA anterior `a0a32a87` #43071798491](https://github.com/franklinbaldo/ies/actions/runs/43071798491) **7/7 PASS** com replay real de cassettes originais; 🦅 LGTM técnico independente nesse SHA. [CI rápido #43080757599](https://github.com/franklinbaldo/ies/actions/runs/43080757599) **PASS** no HEAD novo; [CI oficial #43080806932](https://github.com/franklinbaldo/ies/actions/runs/43080806932) **IN_PROGRESS** ao conferir. Não autorizar squash antes de ver sete gates e replay no HEAD exato.
- **Proveniência:** [card UnB documento-escritas](https://github.com/franklinbaldo/ies/blob/main/cassettes/unb/documento-escritas-unb--36a904749e20.md), ZIP SHA-1 `c7a4af8d9d70ddc3599cdd9bccbb98e6d726db78`, membro LF SHA-256 `36a904749e2042c769cb37034f2e21f05b1c6c02432ac458f6f8bad902d04c02`. Esses hashes são do card. O CI anterior executou `scripts/replay acervo` (34 acervo e 18 cassette PASS), mas não capturou `Error::source()` no HTTP.
- **Limites:** regressão Rust de causa tem RED/GREEN histórico; IES↔cassette e anti-IES↔cassette **PASS apenas nos leitores gerais anteriores**, upload específico **NOT_RUN**; CLI real↔anti-IES↔SQLite **NOT_RUN/não aplicável** por mudança exclusivamente diagnóstica; treinamento/live **NOT_RUN**. Nenhuma escrita externa.
- **Handoff:** [comentário #6804388727](https://github.com/franklinbaldo/ies/pull/778#issuecomment-6804388727). Evitar retrabalho: sem nova modificação documental nesta PR; esperar CI oficial/revisão focal e encaminhar squash a 🐘/🦡. Se main avançar só por memórias disjuntas após aceite, não promover churn gratuito; se o HEAD mudar, revalidar obrigatoriamente.
