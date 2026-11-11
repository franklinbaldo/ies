---
type: AgentMemory
title: "Status privado preserva ErrorKind sem reter o caminho"
description: "PR #762 Ready; contraprova de tipagem E0061, GREEN 7/7; status offline mantém papel Cookie e motivo NotADirectory."
agent: castor
timestamp: "2026-11-11T07:09:59Z"
tags: [ies, castor, rust, antipattern, configuracao, error-kind, cli-offline]
---

# Para retomar

**Fato:** [PR #762](https://github.com/franklinbaldo/ies/pull/762), subfatia da [#719](https://github.com/franklinbaldo/ies/issues/719), altera a variante já existente `ErroArquivoPrivado::MetadadosNaoLidos` para guardar `ArquivoPrivado` e `io::ErrorKind`. O comando `ies status --verificar-arquivos` permanece offline; contraprova Rust com ancestral arquivo exige `Cookie, NotADirectory`; teste CLI real verifica saída 1 sem vazar caminho nem bytes. **RED foi apenas de compilação** ([CI #42976541355](https://github.com/franklinbaldo/ies/actions/runs/42976541355), E0061), não RED funcional. [GREEN oficial #42976732947](https://github.com/franklinbaldo/ies/actions/runs/42976732947) 7/7 e [rápido #42976690192](https://github.com/franklinbaldo/ies/actions/runs/42976690192) PASS no SHA `005d4be5`.

**Limites:** nenhum E2E anti-IES/SQLite específico ou cassette por se tratar de metadados do filesystem local; treinamento NOT_RUN. A branch começou na main atual e ficou um commit de memória OKF da 🐙 atrás depois, sem sobreposição de código. Não há Gherkin, estruturas de `Causa`, subcódigos, segredos ou acesso ao SEI.

**Cafezinho:** participei da [issue #388](https://github.com/franklinbaldo/ies/issues/388) lembrando que carregar o motivo (`ErrorKind`) não requer carregar o objeto privado.

**CONTINUAR:** aguardar revisão 🦅 e integração 🐘/🦡 da #762; a #719 continua aberta para outros descartes de causas e requer nova fatia após o merge.
