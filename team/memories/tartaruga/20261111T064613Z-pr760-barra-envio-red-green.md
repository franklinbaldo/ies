---
type: AgentMemory
title: "PR #760: recusa tipada da barra de envio não depende da unidade"
description: "Fatia #720 com RED específico e GREEN 7/7, inspeção de duas cassettes (77 HTTP), sem confronto semântico da anomalia."
agent: tartaruga
timestamp: "2026-11-11T06:46:13Z"
tags: [issue720, processo-enviar, barra, erros-tipados, cassettes, red-green, ci]
---

# Retomada

**FEITO:** [PR #760](https://github.com/franklinbaldo/ies/pull/760) muda um `match` em `src/processo_enviar/fluxo.rs`: antes `(Ok(None) | Err(_), false)` perdia `ErroEnviarNaBarra` quando a unidade da sessão não figurava na situação; agora `Err(erro)` sempre sobe como `ErroEnviar::Barra(erro)`. Duas contraprovas em `tests_fluxo.rs` abrangem link duplicado e de outro processo, com recusa antes da tela/POST. Sem novos enums ou subcódigos.

**RED comprovado:** [CI #42973712698](https://github.com/franklinbaldo/ies/actions/runs/42973712698) no head `c2ccfb38`, 2082 testes PASS, um FAIL precisamente `ProcessoNaoAbertoNaUnidadeDaSessao` vs `Barra(LinksDeEnviarRepetidosNaBarra)`, 44 ignored. **GREEN:** [CI completo #42974235450](https://github.com/franklinbaldo/ies/actions/runs/42974235450) **7/7** e [rápido #42974169271](https://github.com/franklinbaldo/ies/actions/runs/42974169271) PASS no SHA `ac440fdd`; PR Ready e behind=0 imediatamente antes desta memória.

**Cassette-first:** [workflow oficial #42973645501](https://github.com/franklinbaldo/ies/actions/runs/42973645501) validou ZIP SHA-1 e JSON SHA-256 das cassettes `processo-escritas-unb--47c0acc994ac` e `processo-escritas-go--841b0341a184`; 77 interações HTTP (56 GET, 21 POST). As cassettes não capturam a combinação de barra inválida + unidade diferente. Resultado Rust **PASS sintético**; novo E2E do binário IES ↔ anti-IES ↔ SQLite e confronto bilateral da anomalia **NOT_RUN**; treinamento **NOT_RUN**. Arquivo temporário do Actions removido da PR.

Visitei o [Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388) e li os comentários recentes. Handoff em [#720](https://github.com/franklinbaldo/ies/issues/720#issuecomment-6789052481). **CONTINUAR:** revisão 🦅 e integração 🐘/🦡; a publicação desta memória na `main` pode deixar a branch um commit atrás, então sincronizar por merge verdadeiro e revalidar os gates antes da integração. #720 segue aberta para outras ablações.
