---
type: AgentMemory
title: "BDD-581-02: preservar efeitos parciais e não perder contratos alheios"
description: "PR #693: RED real de anotacao_depois nulo, GREEN 7/7 no head anterior; merge verdadeiro com main restaurou contrato bloco.incluir que faltava na branch e preservou teste TLS/SQLite."
agent: raposa
timestamp: "2026-11-10T18:44:40Z"
tags: [ux, bdd-581-02, anti-ies, sqlite, sincronizacao, cafezinho]
---

# Para a próxima rodada

Na [PR #693](https://github.com/franklinbaldo/ies/pull/693), o [RED oficial](https://github.com/franklinbaldo/ies/actions/runs/42874628977) mostrou `anotacao_depois:null` apesar do POST de limpeza e GET que confirmou o campo vazio. A correção guarda `Some(false)` na variante tipada de recusa pós-POST. O [CI oficial do head anterior](https://github.com/franklinbaldo/ies/actions/runs/42877020019) deu 7/7; **não herdar** esse resultado para o head reconciliado.

Durante a reconciliação, a comparação revelou uma remoção colateral de `contracts/cli/bloco-incluir.md` e sua referência, alheias à #581. Fiz [merge de dois pais](https://github.com/franklinbaldo/ies/commit/247748896b9163814c008fda9aa77604a0e9ef8b) com árvore da `main` mais somente os nove arquivos da fatia. Assim, o contrato unificado da #692 permanece intacto. Retomar gates do SHA final e solicitar revisão do 🦅.

No [Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6775867924), o lembrete de que evento confirmado não é apagado por falha posterior serve também para integração: um contrato já publicado não deve desaparecer ao trazer uma fatia isolada. **Evidência:** somente E2E sintético; cards UnB/GO hashes PASS (77), paridade semântica e treinamento NOT_RUN.
