---
type: AgentMemory
title: "Ablação de erro local integrada; teste Web antes vazio agora exige recusa"
description: "A #785 entrou por squash com replay do acervo verde no SHA; a #790 torna observável o falso teste de login e aguarda seu CI oficial."
agent: texugo
timestamp: "2026-11-11T12:56:00Z"
tags: [antichurn, ablacao, cassette-e2e, github-actions, testes]
---

# Retomar

Li integralmente `AGENTS.md` e `team/texugo.md`, as cinco memórias recentes e o guia de cassettes; visitei o [Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388), que tinha 182 comentários.

**Fato:** integrei a [#785](https://github.com/franklinbaldo/ies/pull/785) por squash no commit [b96809d](https://github.com/franklinbaldo/ies/commit/b96809d2cc352e4d05ac7fc4901b4f34a62a7a21), após verificar [CI #43020611076](https://github.com/franklinbaldo/ies/actions/runs/43020611076), 7/7 PASS com replay original no SHA `9edd5593`, revisão 🦅 favorável e main posterior alterada somente por três memórias OKF sem colisão de código. É regressão local de `NotADirectory`, não paridade com SEI externo.

**Fato:** abri a [#790](https://github.com/franklinbaldo/ies/pull/790) no head `b4791604`, substituindo um teste Rust literalmente vazio de `src/status_web.rs` por negativa sobre formulário de login. Corrigi o nome que antes alegava HTTP 200 sem receber status. [CI rápido #43024336032](https://github.com/franklinbaldo/ies/actions/runs/43024336032) PASS; disparei CI oficial por Draft→Ready, [run #43024492362](https://github.com/franklinbaldo/ies/actions/runs/43024492362) inicialmente na fila. **Não mesclar sem Rust e replay do acervo PASS no HEAD final.** O HTML da negativa é sintético, não cassette de transição nem live.

**Bloqueio verificável:** a [#781](https://github.com/franklinbaldo/ies/pull/781) teve [CI #43023498443](https://github.com/franklinbaldo/ies/actions/runs/43023498443) 7/7, mas a revisão 🦅 identificou ausência da execução nominal do novo `unidade_trocar::acervo::replay_de_unidade_trocar_no_acervo`; portanto o replay pertinente ainda é NOT_RUN, não liberar só pelo número de jobs. #777 e #778 tinham CI oficial em execução.

**Próxima rodada:** verificar #790 no SHA exato; se FAIL, ler logs e corrigir apenas a branch própria. Reavaliar #781 quando o replay específico de Unidade Trocar executar no Actions. Priorizar #716–#723, não contar GET/POST como paridade nem tocar branches alheias.
