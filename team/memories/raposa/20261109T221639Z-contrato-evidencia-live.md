---
type: AgentMemory
title: "Concordância causal ausente na UnB: decisão e contrato atualizados"
description: "A #605 já estava integrada, mas contrato/decisão #264 ainda diziam que a confirmação live era NOT_RUN; PR #610 corrige o registro sem executar novo POST."
agent: raposa
timestamp: "2026-11-09T22:16:39Z"
tags: [ux, atribuicao, evidencia, contrato, unb, cassette-first, causalidade]
---

# Retomar

**Fato:** [PR #605](https://github.com/franklinbaldo/ies/pull/605) integrada com CI 7/7 documenta prévia PASS, POST único na UnB de treinamento e saída 3 apesar de `atribuicao_relida=aluno2`; possível resíduo, remoção NOT_RUN, sem repetir escrita. O contrato e a decisão ainda tinham afirmações históricas de live NOT_RUN ou confirmação sempre fechada. [PR #610](https://github.com/franklinbaldo/ies/pull/610) atualiza apenas essas duas fontes OKF, distinguindo a UnB das demais origens; CI rápido PASS e oficial em verificação ao gravar esta memória.

**Limite:** sem CLI, anti-IES, SQLite ou SEI executados nesta fatia. As 28 telas GET confrontadas e 14 respostas POST caracterizadas não provam causalidade. Próximo passo: revisão/CI/integração #610 e investigação da identidade de aceite a partir das provas preservadas. Nunca repetir o POST incerto para forçar sucesso.

**Cafezinho:** a [conversa sobre hashes e gabarito](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6752736877) lembra que integridade de cassette e confronto funcional são réguas distintas. É inspiração, não prova adicional.
