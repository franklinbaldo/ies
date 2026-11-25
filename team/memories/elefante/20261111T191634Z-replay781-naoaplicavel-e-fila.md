---
type: AgentMemory
title: "Replay #781 agora rejeita NaoAplicavel; sincronização e CI pendentes"
description: "HEAD da #781 passou a exigir Resultado::Lido, inclusive contra falso verde NaoAplicavel; CI oficial ainda em curso e main avançou por memória OKF."
agent: elefante
timestamp: "2026-11-11T19:16:34Z"
tags: [integracao, replay, cassettes, ablação, ci, fila, cafezinho]
---

# Retomar na próxima rodada

- **Fato:** li integralmente `AGENTS.md`, ficha 🐘, cinco memórias OKF e o [Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388). Conferi diffs, revisões, contratos, cards e os sete jobs GitHub Actions. Nenhuma PR integrada por mim nesta rodada.
- **#781:** HEAD `fc4d7ca424afc6481ffd3ec0891830d02c32e2fa`. A alteração pós-`c002765d` endurece a regressão de `unidade trocar`: toda leitura pertinente precisa ser `Resultado::Lido`, inclusive recusando `NaoAplicavel`. Os cards UnB/GO têm SHA-1 e SHA-256 nos metadados. [CI oficial #43084998342](https://github.com/franklinbaldo/ies/actions/runs/43084998342) estava 6/7, replay amplo em execução, `behind=1` pela memória OKF da Raposa. O [CI #43084371328](https://github.com/franklinbaldo/ies/actions/runs/43084371328) foi cancelado após nova alteração. [Comentário de coordenação](https://github.com/franklinbaldo/ies/pull/781#issuecomment-6805001908).
- **Outras PRs:** #778 HEAD `1422bb53`, 7/7 [#43080806932](https://github.com/franklinbaldo/ies/actions/runs/43080806932), atrás da main; #777 HEAD `eb06db9d`, 7/7 [#43080129451](https://github.com/franklinbaldo/ies/actions/runs/43080129451), atrás, 503 pós-POST só sintético; #792 HEAD `adac3b16`, 6/7 [#43082566668](https://github.com/franklinbaldo/ies/actions/runs/43082566668), replay pendente e atrás. #526 e #784 Draft.
- **#796:** já integrada por outra sessão, [commit `8af03222`](https://github.com/franklinbaldo/ies/commit/8af03222c5236f4bf2c29719ade6968c0d336678). O falso aceite de `acao=` duplicada que eu havia identificado recebeu RED e correção segundo [comentário da PR](https://github.com/franklinbaldo/ies/pull/796#issuecomment-6803683045).
- **Coordenação:** [roadmap #56](https://github.com/franklinbaldo/ies/issues/56#issuecomment-6805012106) atualizado. Revalidar estado atual (nunca herdar PASS antigo), revisão, replay nominal, 7/7 e `behind=0` antes de squash. As memórias na main podem reiniciar a janela de sincronização; não contornar a regra. Sem BDD/Gherkin, novos subcódigos, Jatobá, segredos, produção ou escrita externa.
