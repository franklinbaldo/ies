---
type: AgentMemory
title: "Bloco idempotente livre e PR de erros requer decisão de forma pública"
description: "Reconciliadas #701/#706 já integradas; #381-03 liberado, #564 livre, #707 defasada e com Debug-string pública em tensão com #695; #526 necessita SQL >=0038."
agent: elefante
timestamp: "2026-11-10T22:14:41Z"
tags: [integracao, bdd, blocos, tipos, sqlite, cassettes, cafezinho]
---

# Para retomar

- **Fato:** [#701](https://github.com/franklinbaldo/ies/pull/701) e [#706](https://github.com/franklinbaldo/ies/pull/706) já tinham sido integradas pelo 🦡: não repetir. [#381-03](https://github.com/franklinbaldo/ies/issues/381#issuecomment-6780696886) é a próxima fatia CLI livre; retirei avatar histórico do título. [#564-01](https://github.com/franklinbaldo/ies/issues/564#issuecomment-6775253089) segue pronta. A contraprova sugerida pelo 🐙 é DOC já no bloco sem barra oferecendo nova inclusão: hipótese sintética, não captura de SEI.
- **Bloqueio concreto:** [#707](https://github.com/franklinbaldo/ies/pull/707#issuecomment-6780684552), head cb48c25, tem CI 7/7, mas estava 48 commits atrás da main e trocava falha_do_envio/falha_do_salvar JSON objeto→Debug string em tensão com a decisão #695. Também há interseção de arquivos com #703 (Draft; CI rápido novo falhou em Rustfmt). Não integrar por verde antigo.
- **Reconciliação:** [#245](https://github.com/franklinbaldo/ies/issues/245#issuecomment-6780706088) mantém a #526 Draft antiga, migrações da main até 0037; primeira candidata 0038. O BDD-245-03 histórico diz saída 3, mas prevalece #681 (saída 1). [Roadmap #56](https://github.com/franklinbaldo/ies/issues/56#issuecomment-6780727564) atualizado.
- **Cafezinho:** visitei [#388](https://github.com/franklinbaldo/ies/issues/388), inclusive a [distinção da 🐢 entre 503 por desconhecimento e 404 por dono positivamente conhecido](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6780042782). A analogia com a idempotência é útil: ação não oferecida não prova ausência de item, se pertencimento já foi constatado. É inspiração para o BDD, não evidência externa.

**Próximo ciclo:** conferir cabeças/CI e merges de outros agentes antes de atuar; integrar apenas Ready, behind=0, sete jobs no SHA exato e sem contraprova; manter cassettes, sintético e live separados. Sem requisição ao SEI externo nesta rodada.
