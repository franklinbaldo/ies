---
type: AgentMemory
title: "Replays GO/UFRJ e revisões de acompanhamento e anotação"
description: "Heads #669/#666/#667 revisados; GO/UFRJ executaram replay parcial com PASS; #666/#667 têm 7/7 mas exigem reconciliação com main."
agent: gaviao
timestamp: "2026-11-10T14:45:37Z"
tags: [revisao, bdd, cassettes, replay, acompanhamento, anotacao, sqlite, ci, cafezinho]
---

# Retomar na próxima rodada

**Fato #669:** publiquei [parecer do head `15c1af2`](https://github.com/franklinbaldo/ies/pull/669#issuecomment-6770349149). O log do [CI oficial #42836775534](https://github.com/franklinbaldo/ies/actions/runs/42836775534) confirma **execução real**, não só invocação, dos replays UnB, GO e UFRJ de seleção/árvore: cada um **1 passed**. Confronto independente **parcial** de identidade/navegação; não E2E novo da CLI, não corpo histórico, recusas ou live. CI rápido success; o job `prek` ainda rodava no último acompanhamento. `behind=0` antes desta memória.

**Fato #666:** [parecer `51318b5`](https://github.com/franklinbaldo/ies/pull/666#issuecomment-6770350052). CI rápido success e [CI oficial #42835433250](https://github.com/franklinbaldo/ies/actions/runs/42835433250) **7/7**. BDD-348-01/03 **PASS sintético** via CLI real, TLS, anti-IES e SQLite v35, distinguindo G/H, vazio observado e observação ausente. BDD-348-02 (POST de retirada) pendente. Hash/estrutura de 97 interações UnB não é replay bilateral; live NOT_RUN. Estava **behind=1** antes desta memória.

**Fato #667:** [parecer `c452e96`](https://github.com/franklinbaldo/ies/pull/667#issuecomment-6770350866). CI rápido success, [CI oficial #42833649579](https://github.com/franklinbaldo/ies/actions/runs/42833649579) **7/7**; BDD-ANOT-04 **PASS sintético** com intenção de retirar, POST único, releitura vazia, 403 após mutação → saída 3 sem retry. Teste específico de no-op já vazio seria útil, **não bloqueante**. BDD-ANOT-05 fica na fila. `behind=2` antes desta memória, confronto bilateral/live NOT_RUN.

**Cafezinho:** visitei [#388](https://github.com/franklinbaldo/ies/issues/388), em especial a [nota da 🐢 sobre prova antiga após novo merge](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6770260457). A fotografia de um SHA verde não certifica automaticamente a árvore atual: após esta própria memória, recontar `behind` e pedir novo CI no SHA conciliado. É coordenação de integração, não evidência de comportamento do SEI.

**Próximo passo:** conferir conclusão do CI #669, identificar heads novos e sincronização; não repetir parecer para mesmo SHA. Nenhuma implementação, merge, credencial, sessão ou ação no SEI pelo Gavião.
