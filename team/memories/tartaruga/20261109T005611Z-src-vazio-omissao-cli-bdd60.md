---
type: AgentMemory
title: "BDD-60-05: conteúdo sem src testado pela CLI real"
description: "PR #522 integrada; CLI real contra anti-IES/SQLite distingue documento existente com src vazio, recusa ou omite por opção; CI 7/7 e cassette UnB conferida estruturalmente."
agent: tartaruga
timestamp: "2026-11-09T00:56:11Z"
tags: [anti-ies, documento, navegacao, bdd, cli, cassette, sqlite]
---

# Para retomar

**Fato:** a [PR #522](https://github.com/franklinbaldo/ies/pull/522) foi integrada na main por [merge a4fad83](https://github.com/franklinbaldo/ies/commit/a4fad83df1b40bbe07e3de55489ee04333fd7494). O novo teste BDD-60-05, no binário `ies`, usa TLS efêmero em loopback, servidor anti-IES e SQLite compartilhado: documento sem `Nos[i].src` **existe** na árvore e causa recusa tipada; com `--ocultar-inacessiveis`, a CLI omite somente esse documento, conserva o legível e contabiliza requisições. Para caber no limite do projeto, suporte TLS/SQLite foi extraído para `tests/support/documento_interprocessos.rs` sem criar camada produtiva. O [CI completo final](https://github.com/franklinbaldo/ies/actions/runs/42542463193) aprovou **7/7** e o [CI rápido](https://github.com/franklinbaldo/ies/actions/runs/42542451419) passou no head `2f9b313`.

**Cassette-first:** [Actions #42541934281](https://github.com/franklinbaldo/ies/actions/runs/42541934281) conferiu SHA-1, SHA-256, schema e estrutura de `documento-numero-e-busca` da UnB (**15 interações**: 10 GET, 5 POST); marcador temporário removido. **Prova sintética da CLI PASS**, **integridade estrutural da cassette PASS**, **confronto independente IES/anti-IES ↔ respostas originais NOT_RUN**, **live NOT_RUN** e **REA NOT_RUN**. Erros intermediários de Rustfmt e Clippy foram problemas de qualidade e foram corrigidos, **não** RED funcional. Não houve acesso ao SEI de produção, novos subcódigos, credenciais ou TLS relaxado.

**Continuidade:** issue [#60](https://github.com/franklinbaldo/ies/issues/60) permanece aberta. Próximo trabalho deve verificar antes se BDD-60-01/02 já têm prova funcional na main, sem reimplementar fatias integradas; se houver lacuna, começar por card/cassette e CLI real. Visita ao [Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388): a distinção discutida entre ausência de observação e observação do vazio é uma analogia útil para `src` ausente, **não** evidência do SEI.
