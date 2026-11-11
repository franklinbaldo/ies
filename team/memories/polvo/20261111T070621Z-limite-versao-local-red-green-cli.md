---
type: AgentMemory
title: "Versão local máxima: RED/GREEN da CLI sem saturação"
description: "PR #761 protege a prévia offline no limite u32::MAX; RED do binário real, GREEN 7/7 no head reconciliado. Sem confronto bilateral/live."
agent: polvo
timestamp: "2026-11-11T07:06:21Z"
tags: [documentos, versoes, ablacao, cli, red-green, cassette-e2e]
---

# Retomar

- **Fato:** [PR #761](https://github.com/franklinbaldo/ies/pull/761) corrige `copia_documento::par`: `saturating_add(1)` reutilizava `DOC.v4294967295.html` como própria edição, reportando `EdicaoIgualABase` em vez de `VersaoEditadaAusente`. Substituído por `checked_add(1)`, com teste E2E do binário CLI real offline e contrato atualizado. Sem novo subcódigo, tipo de causa ou mudança Web.
- **Evidência:** RED test-only no [CI #42975433877](https://github.com/franklinbaldo/ies/actions/runs/42975433877) falhou exatamente no erro esperado; [CI completo #42976339157](https://github.com/franklinbaldo/ies/actions/runs/42976339157) **7/7 SUCCESS** e [rápido #42976325021](https://github.com/franklinbaldo/ies/actions/runs/42976325021) PASS no head `72c764c`, reconciliado por merge verdadeiro. PR Ready, mergeable; main avançou depois apenas em memória de Texugo, sem sobreposição.
- **Limites:** cards UnB de versões e escritas consultados antes de criar a fixture, sem versão máxima observada; ZIP não reinspecionado nesta fatia. Contraprova é sintética CLI real offline, não replay. E2E Web IES↔anti-IES↔SQLite novo NOT_RUN (não aplicável à alteração local); confronto semântico bilateral NOT_RUN; treinamento/live NOT_RUN. Nenhum acesso SEI.
- **Cafezinho:** visitei a [#388](https://github.com/franklinbaldo/ies/issues/388). A distinção da 🦫 sobre preservar uma regressão mesmo após mudar o método de testes foi útil para manter o E2E Rust, sem fabricar Gherkin.
- **Continuação:** revisão independente 🦅, integração 🐘/🦡 da #761, e depois nova fatia livre #716–#723. Não disputar branches de colegas nem fechar #722 enquanto houver ocorrências.
