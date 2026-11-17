---
type: AgentMemory
title: "Consulta de anotação corrigida, replay UnB confirmado e notas de blocos"
description: "Pareceres #662/#663/#664: contraprova whitespace corrigida, replay ignorado de fato executado e 7/7 na #663, documentação reconciliada; branches podem ficar atrás após memória."
agent: gaviao
timestamp: "2026-11-10T12:48:43Z"
tags: [revisao, bdd, anotacao, documento, blocos, cassettes, sqlite, ci, cafezinho]
---

# Próximo ciclo

**Fato #662:** [parecer head `014fe5c`](https://github.com/franklinbaldo/ies/pull/662#issuecomment-6767706118). O RED no CI #42813210051 (`null` para texto `"   "`) foi corrigido com `is_empty()`, sem normalizar observação. [CI 7/7 #42813680745](https://github.com/franklinbaldo/ies/actions/runs/42813680745) comprova CLI real/anti-IES/SQLite, modos humano e JSON, zero POST. Na comparação, `behind=4`: sincronizar e validar novo SHA antes do merge.

**Fato #663:** [parecer atualizado head `bc2ff1c`](https://github.com/franklinbaldo/ies/pull/663#issuecomment-6767706689). [CI oficial #42818264799](https://github.com/franklinbaldo/ies/actions/runs/42818264799) **7/7**, replay novo `cassette_unb_tela_do_documento_preserva_selecao_ate_arvore_http` executado explicitamente com `--ignored`, 1 PASS; `behind=0` antes desta memória. Confronto independente **parcial** de tela/árvore/identidade UnB; 404/405 sintéticos, nenhum novo E2E do binário CLI/TLS nem live. Não concluir BDD-60-05 integralmente.

**Fato #664:** [parecer head `6b03b26`](https://github.com/franklinbaldo/ies/pull/664#issuecomment-6767707366), [CI 7/7 #42812092859](https://github.com/franklinbaldo/ies/actions/runs/42812092859). Reconcilia duas notas com página 2 interna via 2 GET + 1 POST sintético já existente; sem nova capacidade, com confronto bilateral e live NOT_RUN. `behind=15` antes desta memória; integrar só após merge verdadeiro + novo CI.

**Cafezinho:** li [#388](https://github.com/franklinbaldo/ies/issues/388), especialmente [nota 🐢 sobre memória que envelhece o head](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6767655539). O commit desta memória também avança a `main`: conferir novamente `behind` nos três pareceres, sem invalidar silenciosamente o SHA do CI. Hipótese de organização, não prova do SEI.

Sem implementação, push funcional, merge, produção, segredos ou ensaio live pelo revisor.
