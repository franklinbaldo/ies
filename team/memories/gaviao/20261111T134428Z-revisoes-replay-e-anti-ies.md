---
type: AgentMemory
title: "Cinco pareceres: replay nominal, ablações e oráculo independente"
description: "Pareceres por SHA #792, #790, #781, #782 e #777. Replays oficiais dos três primeiros e do #777 aprovados; #782 ainda sem CI completo; paridade circular de anexação segue pendente."
agent: gaviao
timestamp: "2026-11-11T13:44:28Z"
tags: [revisao, cassette-first, anti-ies, ci, paridade, antipatterns, cafezinho]
---

# Retomar

- [#792 `aede9c0`](https://github.com/franklinbaldo/ies/pull/792#issuecomment-6797472702): `corpo()` não deve converter base64 corrompido em corpo vazio. CI [#43026498877](https://github.com/franklinbaldo/ies/actions/runs/43026498877) 7/7 com `scripts/replay acervo`, 495 cards e 29 ZIPs, 24 testes acervo e 18 cassette_. Corrupção não capturada; `null` ainda equivale a ausência (atenção não bloqueante). `behind=3`.
- [#790 `b479160`](https://github.com/franklinbaldo/ies/pull/790#issuecomment-6797473483): substituído teste vazio por negativa Rust real para login. [CI #43024492362](https://github.com/franklinbaldo/ies/actions/runs/43024492362) 7/7 com replays gerais; teste sintético, login externo NOT_RUN. `behind=5`, `mergeable=false`.
- [#781 `c76f8e8`](https://github.com/franklinbaldo/ies/pull/781#issuecomment-6797474209): após incorporar `main`, [CI #43026601119](https://github.com/franklinbaldo/ies/actions/runs/43026601119) 7/7 executou **nominalmente** replay de troca de unidade, 32 cassettes, 22 Controles novos, nenhuma recusa. O antigo falso verde de relatório foi corrigido. Bilateral anti-IES NOT_RUN nesta fatia de testes; `behind=3`.
- [#782 `e7b1a04`](https://github.com/franklinbaldo/ies/pull/782#issuecomment-6797475029): refatoração transversal de 88 arquivos, agora usa `processo_anexar/acervo_leitores.rs` para replay IES. No head há **somente CI rápido** [#43028603671](https://github.com/franklinbaldo/ies/actions/runs/43028603671); exigir replay nominal no CI completo final e preservar os dois lados ao juntar com #777. `behind=3`.
- [#777 `cb147ae`](https://github.com/franklinbaldo/ies/pull/777#issuecomment-6797475788): [CI #43024637308](https://github.com/franklinbaldo/ies/actions/runs/43024637308) 7/7 executou 10 leituras GET originais e 10 respostas anti-IES, mas o banco é semeado por `ler_tela` do **próprio IES**. A igualdade de IDs não é oráculo independente. Requer contraprova por dados extraídos separadamente da captura; HTTP 503 pós-POST continua sintético, live NOT_RUN. `behind=4`.

[Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388): li a conversa recente sobre a diferença entre testes que só imprimem tabelas e testes que falham de fato, e a anotação do 🦫 sobre descartar `body_base64` inválido. Não é prova externa.

Sem editar branches alheias, implementação, merge, produção, segredo, retry ou Gherkin. Próxima rodada: revisar **somente SHAs novos**, verificar CI/replay no SHA final das branches reconciliadas e reavaliar a não circularidade dos dados de anexação.
