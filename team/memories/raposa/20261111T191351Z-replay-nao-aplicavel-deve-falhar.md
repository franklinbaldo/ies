---
type: AgentMemory
title: "Replay de troca: NaoAplicavel também deve reprovar"
description: "PR #781 sincronizada com main e gate endurecido contra resposta ilegível; rápido PASS no fc4d7ca4, replay oficial ainda em execução."
agent: raposa
timestamp: "2026-11-11T19:13:51Z"
tags: [ux, unidade-trocar, cassette-first, replay, ci, cafezinho]
---

# Para retomar

**Fatos:** reli `AGENTS.md`, `team/raposa.md`, as cinco memórias recentes e visitei o [Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388). O CI anterior [#43075678387](https://github.com/franklinbaldo/ies/actions/runs/43075678387) foi 7/7 PASS no SHA `9f80d867`, com replays originais de 32 cassettes de troca. Sincronizei a [PR #781](https://github.com/franklinbaldo/ies/pull/781) por merge verdadeiro `c002765d` com `main@36ed2b91`, mantendo diff só dos dois testes Rust.

**Achado:** a asserção recém-adicionada só reprovava `Recusado`, mas uma falha de decodificação do Controle pós-POST virava `NaoAplicavel("corpo ilegível")` e continuava elegível ao verde. No [commit `fc4d7ca4`](https://github.com/franklinbaldo/ies/commit/fc4d7ca424afc6481ffd3ec0891830d02c32e2fa), exigi `Resultado::Lido` em **todas as leituras avaliadas**. O [CI rápido #43084976947](https://github.com/franklinbaldo/ies/actions/runs/43084976947) PASS; [CI oficial #43084998342](https://github.com/franklinbaldo/ies/actions/runs/43084998342) em execução ao fechar esta nota. Handoff atualizado na [#721](https://github.com/franklinbaldo/ies/issues/721#issuecomment-6804893238).

**Limites:** IES/leitor↔cassettes originais é PENDENTE neste novo HEAD. Anti-IES↔original, CLI↔anti-IES↔SQLite para troca e live: NOT_RUN. Os dez desvios pós-POST são sintéticos. Não confundir `NaoAplicavel` com confirmação. A conversa do 🦫 no Cafezinho sobre `body_base64` corrompido destaca risco semelhante de aceitar ausência como leitura válida.

**CONTINUAR:** conferir 7/7 PASS e execução nominal do replay no SHA `fc4d7ca4`; pedir revisão focal 🦅, integração por 🐘/🦡, sem novos merges memoriais que invalidem o gate.
