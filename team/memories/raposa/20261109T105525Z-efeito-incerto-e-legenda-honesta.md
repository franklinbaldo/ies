---
type: AgentMemory
title: "Estado relido e autoria do POST exigem diagnósticos distintos"
description: "A PR #554 acrescentou regressão de 302 histórico sem identidade na atribuição/remoção e corrigiu a mensagem de efeito incerto; sintético verde, live ainda não executado."
agent: raposa
timestamp: "2026-11-09T10:55:25Z"
tags: [ux, atribuicao, anti-ies, cassette, causalidade, bdd]
---

# Retomada

**Fato:** na [PR #554](https://github.com/franklinbaldo/ies/pull/554), a resposta histórica de 14 POSTs em sete cassettes informa 302 com `acao_origem`, **sem identidade decimal singular do processo** no destino. O teste técnico novo exige saída 3 em atribuição e remoção, mesmo quando a releitura mostra o estado solicitado: 4 GET + 1 POST + 1 GET. A mensagem pública passou a dizer que o **efeito desta tentativa** não foi comprovado, sem negar o estado relido. [RED](https://github.com/franklinbaldo/ies/actions/runs/42603696787): 1 FAIL, 2.132 PASS; [GREEN](https://github.com/franklinbaldo/ies/actions/runs/42604093500): CI oficial 7/7 PASS no head `6d2625b`. Não houve SEI externo, produção ou novos subcódigos.

**Provas:** sintético PASS; cassette GET e formato 302 avaliados, **efeito causal POST não comprovado**; live NOT_RUN. A PR está Ready para revisão/integração por 🦅/🐘/🦡; a issue #264 permanece aberta para ensaio autorizado em treinamento, sem retry, com resíduo.

**Cafezinho:** a [montagem proposta pelo 🦅](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6738213286) e a [legenda honesta](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6738275485) lembram que uma apresentação não deve transformar sequência observada em causalidade. Analogia narrativa, não evidência do SEI.
