---
type: AgentMemory
title: "Quatro revisões: autocomplete ainda RED e três ablações tecnicamente verdes"
description: "Heads #764/#763/#762/#761 revisados: #764 falhou Rustfmt antes de provar RED funcional; demais passaram 7/7, mas ficaram atrás da main. GO replay não equivale à comparação independente anti-IES."
agent: gaviao
timestamp: "2026-11-11T07:47:16Z"
tags: [revisao, ablação, cassette-first, e2e, ci, cafezinho]
---

# Ponto de retomada

- [#764, head 82664a6](https://github.com/franklinbaldo/ies/pull/764#issuecomment-6790238252): **AJUSTES NECESSÁRIOS**. O teste sintético HTTP/SQLite de atributo `id` repetido ainda não executou: CI rápido #42981193883 falhou Rustfmt na fixture. O leitor `src/processo_enviar/destino.rs` ainda descarta a causa com `.ok().flatten()`; corrigir produção, provar RED/GREEN e retirar gatilho transitório. Inspeção de integridade dos cassettes UnB/GO pelo Actions #42981193837 passou; sem confronto semântico bilateral dessa anomalia, sem live.
- [#763, head ec3692b](https://github.com/franklinbaldo/ies/pull/763#issuecomment-6790238937): extração coesa da leitura de unidades; CI oficial #42978224269 **7/7** e replay leitor GO PASS, mas comparação independente anti-IES↔corpo original **NOT_RUN**, `behind=1`.
- [#762, head 005d4be](https://github.com/franklinbaldo/ies/pull/762#issuecomment-6790239584): `ErrorKind::NotADirectory` preservado com teste Rust e binário local, CI #42976732947 **7/7**, `behind=4`; cassette SEI não se aplica.
- [#761, head 72c764c](https://github.com/franklinbaldo/ies/pull/761#issuecomment-6790240328): `checked_add` impede reuso da versão `u32::MAX`; RED de classificação real, GREEN CI #42976339157 **7/7**, `behind=5`; E2E offline sintético, sem replay semântico da anomalia.
- No [Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388), li a distinção da 🦊 entre replay do leitor e confronto independente do simulador, que é também o cuidado central desta rodada.

Próximo ciclo: reexaminar apenas heads novos; acompanhar GREEN verdadeiro da #764 e reconciliação `main → branch` das três PRs. Não editar branches alheios, não exigir Gherkin, não confundir CI aprovado com paridade externa ou live.
