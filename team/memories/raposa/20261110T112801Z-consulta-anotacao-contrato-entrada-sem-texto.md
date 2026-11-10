---
type: AgentMemory
title: "BDD-ANOT-03: consulta explícita não é texto vazio"
description: "PR #662 Ready no head 7bf0f093, GREEN oficial 7/7 após RED real 2/5 na CLI por --texto obrigatório. Leitura humano/JSON sem POST via anti-IES/SQLite; cassettes UnB/GO hash PASS, semântica/live NOT_RUN."
agent: raposa
timestamp: "2026-11-10T11:28:01Z"
tags: [ux, cli, processo-anotar, bdd-anot-03, anti-ies, sqlite, cassette-first, red-green, cafezinho]
---

# Retomar

**Fato:** a [PR #662](https://github.com/franklinbaldo/ies/pull/662) implementou a leitura de `ies processo anotar NUP` sem `--texto`, com `anotacao` preenchida ou `null` após formulário explicitamente vazio. O [RED #42806286559](https://github.com/franklinbaldo/ies/actions/runs/42806286559) teve 2 falhas novas e 3 testes anteriores PASS: `clap` recusava `--texto` ausente (exit 2), sem qualquer HTTP. [GREEN #42807219407](https://github.com/franklinbaldo/ies/actions/runs/42807219407) passou 7/7; [CI rápido #42807183002](https://github.com/franklinbaldo/ies/actions/runs/42807183002) passou. A vertical CLI Rust real → TLS local → anti-IES → SQLite mantém estado sem POST; reutiliza o renderizador já existente, sem migração nova.

**Provas separadas:** sintética **PASS** no SHA `7bf0f093`; integridade de 2 cassettes UnB/GO, 77 interações, **PASS** em [#42806531707](https://github.com/franklinbaldo/ies/actions/runs/42806531707), mas confronto semântico bilateral **NOT_RUN**; live/treinamento **NOT_RUN** e nenhuma requisição ou escrita ao SEI. Não inventar PASS externo, nem criar subcódigo novo.

**Cafezinho:** reli a [#388](https://github.com/franklinbaldo/ies/issues/388) e [comentei](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6766086226) que ausência de intenção de escrever (flag não passada) não equivale à anotação vazia (fato observado). A mesma sutileza do campo ausente volta na camada de argumentos, antes do parser HTML.

**CONTINUAR:** revisão 🦅 e integração 🐘/🦡 da PR #662; depois BDD-ANOT-04/05 na #638, e confronto bilateral original com cassettes e live autorizado. A gravação desta memória na main pode deixar a PR um commit atrás, exigir merge verdadeiro e revalidação do SHA.
