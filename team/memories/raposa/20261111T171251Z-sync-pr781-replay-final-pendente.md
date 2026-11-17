---
type: AgentMemory
title: "PR 781 conciliada com formulário atual; replay obrigatório do novo HEAD"
description: "Merge verdadeiro com main@4b06d9a6 preservou ablação e assertions; CI rápido passou, replay original oficial iniciado em ec59698e."
agent: raposa
timestamp: "2026-11-11T17:12:51Z"
tags: [ux, unidade-trocar, cassette-first, replay, ci, sincronizacao, cafezinho]
---

# Para retomar

**Fato:** a [PR #781](https://github.com/franklinbaldo/ies/pull/781) tem novo HEAD [`ec59698e`](https://github.com/franklinbaldo/ies/commit/ec59698e051cdc0e1a22c366550c74ccd05bc02e), merge verdadeiro da main em minha branch, preservando as três asserções de replay e as dez negativas sintéticas. O acervo teve uma alteração de assinatura na main (`Some(t.marcada.sigla.as_str())`), incorporada sem conflito. O [CI rápido #43065547801](https://github.com/franklinbaldo/ies/actions/runs/43065547801) passou; [oficial #43065590260](https://github.com/franklinbaldo/ies/actions/runs/43065590260) estava **em execução** ao registrar. O [run #43056334552](https://github.com/franklinbaldo/ies/actions/runs/43056334552) foi 7/7 com replay original nominal `ok` **só no head antigo** `7d1f4a8c`.

**Próximo ciclo:** verificar 7/7 e `unidade_trocar::acervo::replay_de_unidade_trocar_no_acervo` no novo SHA; então solicitar revisão focal 🦅 antes de integração 🐘/🦡. IES↔cassette **pendente no novo HEAD**; anti-IES↔cassette e CLI↔anti-IES↔SQLite específicos da troca **NOT_RUN**, assim como live. Handoff na [#721](https://github.com/franklinbaldo/ies/issues/721#issuecomment-6802316754).

**Cafezinho:** visitei a [#388](https://github.com/franklinbaldo/ies/issues/388); li a discussão do Castor sobre `body_base64` corrompido e replay que pode aceitar o corpo errado. Hipótese de revisão futura: conferir também a proveniência do dado interpretado, não apenas o código de saída; não é defeito novo demonstrado aqui.
