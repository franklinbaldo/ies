---
type: AgentMemory
title: "Setup com uma consulta de metadados e causa íntegra"
description: "PR #765 com RED funcional 2084/1 e GREEN oficial 7/7; uma metadata por arquivo preserva NotADirectory sem expor caminho."
agent: castor
timestamp: "2026-11-11T08:19:04Z"
tags: [ies, castor, rust, antipattern, setup, error-kind, red-green, e2e-local]
---

# Para retomar

**Fato:** [PR #765](https://github.com/franklinbaldo/ies/pull/765) resolve uma perda real de causa no `ies setup`: `Path::is_file()` silenciava `NotADirectory`, e a segunda consulta de `metadata()` nunca chegava a relatá-lo. Uma única consulta de metadados preserva `ArquivoPrivadoNaoLido(ErrorKind)` já existente, mantendo `NotFound` e arquivo não regular na classificação anterior. Três arquivos alterados, incluindo contraprova Rust e E2E com o binário real sem rede.

**RED funcional:** [CI #42983656491, tentativa 2](https://github.com/franklinbaldo/ies/actions/runs/42983656491/attempts/2), 2.084 Rust PASS e 1 FAIL específica. **GREEN:** [CI #42985323984](https://github.com/franklinbaldo/ies/actions/runs/42985323984) 7/7 PASS, [rápido #42985302800](https://github.com/franklinbaldo/ies/actions/runs/42985302800) PASS, head `80b09f7f`. Merge verdadeiro da main `5e0ce65d`, sem rebase; a main ganhou depois uma memória OKF disjunta.

**Limites:** cassette, anti-IES/SQLite não se aplicam aos metadados locais; treinamento/live NOT_RUN. No [Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6790558753) registrei que uma verificação defensiva pode apagar o diagnóstico. Nenhuma nova Causa, subcódigo ou Gherkin.

**CONTINUAR:** obter revisão 🦅 e integração 🐘/🦡 da #765; a #719 segue aberta para outras perdas comprovadas.
