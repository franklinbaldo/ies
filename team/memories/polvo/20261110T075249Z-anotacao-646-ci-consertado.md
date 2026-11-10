---
type: AgentMemory
title: "Anotação: referência dupla quebrou compilação após correção do Clippy"
description: "BDD-ANOT-01 #646 revalidado após merger de memórias; `&&Erro` corrigido para `&Erro`; CI oficial 7/7, cassette e live ainda parciais."
agent: polvo
timestamp: "2026-11-10T07:52:49.072Z"
tags: [bdd638, anotacao, anti-ies, rust, erro-tipado, ci, cassettes]
---

# Para retomar

- **Fato:** A [PR #646](https://github.com/franklinbaldo/ies/pull/646), BDD-ANOT-01 de [#638](https://github.com/franklinbaldo/ies/issues/638), foi sincronizada à main por merge verdadeiro de dois pais [`99e5854`](https://github.com/franklinbaldo/ies/commit/99e5854c152606272594ac20a72d5f287c3293b9). Quatro arquivos de memória foram preservados, sem alteração concorrente funcional.
- **Contraexemplo:** [CI #42777710543](https://github.com/franklinbaldo/ies/actions/runs/42777710543) falhou na compilação em todos os jobs Rust porque `falha_da_previa(erro: &Erro)` passava `&erro` a `caminho_de(&impl Causa)`, criando `&&Erro`. O compilador pediu `caminho_de(erro)`; o commit [`a3d64d1`](https://github.com/franklinbaldo/ies/commit/a3d64d169b60ca257e4185d16ffeff50acaa2595) aplicou só essa correção.
- **Validação:** [CI oficial #42777849836](https://github.com/franklinbaldo/ies/actions/runs/42777849836) **7/7 PASS** e [rápido #42777838962](https://github.com/franklinbaldo/ies/actions/runs/42777838962) PASS no mesmo SHA; PR permanece aberta e Ready. BDD da prévia com binário IES → TLS loopback → anti-IES → SQLite PASS sintético. Cards UnB/GO (37+40 interações) SHA-verificados previamente, **confronto semântico independente NOT_RUN**; treinamento **NOT_RUN**, zero SEI externo.
- **Próxima fatia, depois de integrar:** BDD-ANOT-02 requer `--confirmar`, um POST de registro de texto, releitura causal e efeito incerto sem retry. No anti-IES atual, o POST `anotacao_registrar` só aceita `txaDescricao` vazio para limpeza. O novo estado precisa ser persistido e comprovado na CLI, sem confundir GET da cassette com prova de POST.
- **Cafezinho:** [#388](https://github.com/franklinbaldo/ies/issues/388), visitado após a entrega; permanece útil a distinção feita pelos colegas entre dado ausente, dado observado vazio e efeito realmente atribuído à escrita.
