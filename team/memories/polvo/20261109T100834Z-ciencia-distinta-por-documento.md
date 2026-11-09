---
type: AgentMemory
title: "BDD-271-02: ciências de dois documentos não se confundem"
description: "PR #553 acrescenta E2E sintético da CLI com duas atividades documentais distintas; CI 7/7 verde, autoria da sessão e confronto de cassettes ainda não comprovados."
agent: polvo
timestamp: "2026-11-09T10:08:34Z"
tags: [ciencia, documento, bdd271, anti-ies, sqlite, cassette]
---

# Para retomar

- **Fato:** [PR #553](https://github.com/franklinbaldo/ies/pull/553) está Ready e mergeável no head `79e8b0f4`. O [CI oficial #42598283278](https://github.com/franklinbaldo/ies/actions/runs/42598283278) passou 7/7 após corrigir Rustfmt/Clippy e incorporar `main` por merge verdadeiro. O teste pela CLI real/TLS local/anti-IES/SQLite confirma duas ciências diferentes em documentos distintos do mesmo processo, sem contaminar a segunda prévia. Não houve RED funcional nesta ampliação de regressão.
- **Observação prévia:** cards UnB/UFRJ com 77 interações, integridade verificada no [Actions #42592112909](https://github.com/franklinbaldo/ies/actions/runs/42592112909). O segundo documento é fixture mínima para lacuna; confronto semântico de duas ciências e live **NOT_RUN**. Não confundir prova sintética com SEI real.
- **Lacuna verificável:** a tabela `ciencia_documental` preserva documento, processo e atividade, mas não o **usuário da sessão** exigido pelo BDD-271-02. Próximo passo: obter evidência observável de autoria antes de modelá-la; revisão 🦅 e integração 🐘/🦡, sem fechar a [issue #271](https://github.com/franklinbaldo/ies/issues/271).
- **Cafezinho:** a [conversa sobre dois fólios e dois selos](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6737272733) serviu de metáfora para distinguir identidade do objeto, atividade e autor. Metáfora não é prova.
