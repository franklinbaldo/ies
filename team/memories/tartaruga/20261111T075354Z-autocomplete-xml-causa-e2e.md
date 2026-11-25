---
type: AgentMemory
title: "Autocomplete do envio: erro de id duplicado preservado no E2E"
description: "PR #764 sobre #718: RED funcional da causa genérica, correção tipada, CI rápido verde e CI completo ainda em execução; duas cassettes com integridade verificada."
agent: tartaruga
timestamp: "2026-11-11T07:53:54Z"
tags: [issue718, autocomplete, processo-enviar, erros-tipados, cassette-first, e2e, ci]
---

# Retomar

**Fato:** [PR #764](https://github.com/franklinbaldo/ies/pull/764), head `6f9f0fe6`, elimina `.ok().flatten()` do leitor XML de `ies processo enviar`, que escondia `ErroAtributoHtml` quando `id` vinha duplicado. A variante específica `FalhaDaConsulta::AtributoDoItemIlegivel` preserva a causa, sem subcódigos ou novas estruturas `Causa`. Há E2E adversarial do binário Rust real sobre HTTPS local/anti-IES/SQLite, com consulta única e recusa antes do POST de envio. A [contraprova RED](https://github.com/franklinbaldo/ies/actions/runs/42980900174) falhou pelo motivo esperado; [CI rápido](https://github.com/franklinbaldo/ies/actions/runs/42982326319) PASS no head sincronizado. O [CI completo](https://github.com/franklinbaldo/ies/actions/runs/42982378423) estava 6/7, com `prek` em execução na última consulta; não presumir GREEN E2E antes de confirmar.

**Cassette-first:** [inspeção oficial](https://github.com/franklinbaldo/ies/actions/runs/42981193837) verificou ZIP SHA-1 e JSON SHA-256 dos cards UnB/GO de escritas, 77 interações (56 GET, 21 POST). Ambos contêm autocomplete normal, não atributo duplicado. Regresso da anomalia é **sintético**, confronto bilateral com captura original **NOT_RUN**, live **NOT_RUN**.

**Continuidade:** branch sincronizada por merge verdadeiro com `main@76444be`; a main avançou em `src/unidade_lupa.rs` durante o CI, sem sobreposição. Confirmar os sete gates no SHA, revisar efeito da nova main antes de integrar, responder à [revisão Gavião](https://github.com/franklinbaldo/ies/pull/764#issuecomment-6790238252), encaminhar ao 🐘/🦡. Handoff em [#718](https://github.com/franklinbaldo/ies/issues/718#issuecomment-6790357617). Cafezinho [#388](https://github.com/franklinbaldo/ies/issues/388) visitado: a observação da 🦊 sobre distinguir replay do leitor de confronto independente permanece pertinente.
