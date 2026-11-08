---
type: AgentMemory
title: "Total paginado de Goiás: observação versus identidade sintética"
description: "PR #517 reconciliou total 27.338 da cassette GO ao E2E sem assumir paridade de HTML ou live; RED 2 falhas, GREEN 7/7."
agent: raposa
timestamp: "2026-11-08T23:25:09Z"
tags: [cli, bdd, anti-ies, sqlite, cassette, paginacao]
---

# Para lembrar na próxima rodada

Fato: [PR #517](https://github.com/franklinbaldo/ies/pull/517) ajustou a bancada do BDD-61-02: total declarado 27.338, observado em `pesquisa-paginas-go#3/#4` no [replay do acervo de 04/11](https://github.com/franklinbaldo/ies/blob/main/docs/validacao-replay/2026-11-04-documento-buscar-acervo.md), substitui total sintético 27; as identidades/documentos da fixture permanecem sintéticos. O RED [#42534462875](https://github.com/franklinbaldo/ies/actions/runs/42534462875) teve exatamente 2 E2Es falhos, ambos por 27 versus 27.338. O GREEN [#42534745545](https://github.com/franklinbaldo/ies/actions/runs/42534745545) passou 7/7 no commit `6f361cd`.

Fato: integridade ZIP anterior e replay histórico de leitor IES **não** significam confronto independente de IES e anti-IES contra corpos originais. Cassette semântico e treinamento seguem NOT_RUN nesta fatia. Retomar pela revisão e integração da PR sem reimplementar os BDD-61 já integrados.

A [conversa do Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6728506457) sobre a diferença entre acorde final e gravação multipista inspirou uma analogia: concordar no total é só uma nota correta; falta provar o restante da pauta. Analogia, não evidência do SEI.