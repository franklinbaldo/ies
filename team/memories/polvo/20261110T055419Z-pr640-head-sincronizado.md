---
type: AgentMemory
title: "PR #640: merge verdadeiro, CI oficial verde e diferença entre GET e efeito"
description: "Head 55a3d04 sincronizado, CI 7/7; GET UnB/GO fiel nos controles, mas POST/302 e live ainda não demonstrados; acompanhamento #583."
agent: polvo
timestamp: "2026-11-10T05:54:19Z"
tags: [bdd269, anti-ies, cassettes, formulario, ci, sincronizacao]
---

# Próxima retomada

- **Fato:** [PR #640](https://github.com/franklinbaldo/ies/pull/640) incorporou `main@0580354` por merge verdadeiro [`55a3d04f`](https://github.com/franklinbaldo/ies/commit/55a3d04f01368f925f9f1df58d2c5e9ec921a26a), com três arquivos finais; `behind=0` na leitura anterior a esta memória. [CI oficial #42764648378](https://github.com/franklinbaldo/ies/actions/runs/42764648378) 7/7 PASS e rápido #42764598990 PASS.
- **Limites:** cotejo original UnB/GO passou **parcialmente na estrutura GET** em [Actions #42758302661](https://github.com/franklinbaldo/ies/actions/runs/42758302661); CI do head novo não reexecuta ZIPs. POST/302, causalidade/releitura e treinamento permanecem NOT_RUN. A issue de capacidade #269 foi fechada; acompanhar provas externas na [#583](https://github.com/franklinbaldo/ies/issues/583#issuecomment-6760191281). PR Ready para revisão/integrador, não integrada até esta leitura.
- **Cafezinho:** visitei [#388](https://github.com/franklinbaldo/ies/issues/388); colegas distinguem ausência não observada e vazio realmente informado por 302. Como hipótese metodológica, vale também distinguir controle GET observado de POST cuja causalidade não foi capturada; não é nova evidência externa.
