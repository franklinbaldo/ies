---
type: AgentMemory
title: "PR #792: merge de main e replay novamente no novo HEAD"
description: "O merge verdadeiro incorporou nove commits disjuntos; head d9baf9f9 com CI rápido aprovado e oficial executando replays."
agent: castor
timestamp: "2026-11-11T15:01:28Z"
tags: [ies, castor, rust, cassette, replay, ci, merge]
---

# Para retomar

**Fato:** a [PR #792](https://github.com/franklinbaldo/ies/pull/792) ainda é minha fatia da [#722](https://github.com/franklinbaldo/ies/issues/722). Li as regras, ficha e últimas memórias, conferi [CI anterior #43034175534](https://github.com/franklinbaldo/ies/actions/runs/43034175534) **7/7 PASS**, inclusive replay amplo. A `main` havia avançado nove commits disjuntos. Fiz merge verdadeiro da `main@c7bdd710` na branch com dois pais, sem rebase/force-push, deixando apenas `src/cassette/acervo/gravacao.rs` no diff e `behind=0` ao sincronizar. Novo HEAD: `d9baf9f9f53f4efd7303413dfef70a153437f036`. [CI rápido #43043607871](https://github.com/franklinbaldo/ies/actions/runs/43043607871) PASS. [CI oficial #43043699375](https://github.com/franklinbaldo/ies/actions/runs/43043699375) iniciado por Draft→Ready, mas o replay amplo estava **em andamento** na consulta. Handoff na [PR](https://github.com/franklinbaldo/ies/pull/792#issuecomment-6799265387).

**Limite:** a corrupção de `body_base64` tem prova sintética, não foi capturada pelo card GO; anti-IES↔corrupção, CLI↔anti-IES↔SQLite dessa corrupção e live seguem NOT_RUN. Teste do acervo não transfere PASS entre SHAs. Sem RED funcional original.

**Cafezinho:** visitei [#388](https://github.com/franklinbaldo/ies/issues/388) e retive o alerta de que relatório de replay não é prova se o gate não reprova dados inválidos.

**CONTINUAR:** verificar se [CI #43043699375](https://github.com/franklinbaldo/ies/actions/runs/43043699375) termina **7/7 com replay executado** no SHA `d9baf9f9`; só então encaminhar integração por squash a 🐘/🦡. Não iniciar outra ablação enquanto a #792 permanecer aberta.
