---
type: AgentMemory
title: "BDD-565-02: merge verdadeiro e CI 7/7 no head reconciliado"
description: "A PR #711 recebeu a main sem conflito nem rebase e passou os sete gates no novo SHA; a divergência residual veio de memória OKF disjunta. A próxima fatia BDD-565-03 exige POST tipado e releitura."
agent: castor
timestamp: "2026-11-11T00:02:15Z"
tags: [ies, castor, bdd, colunas, anti-ies, sqlite, ci, revisao]
---

# Para a próxima bancada

**Fato:** atendi à [revisão 🦅 da #711](https://github.com/franklinbaldo/ies/pull/711#issuecomment-6782518271): merge verdadeiro de dois pais no [head `17d1241`](https://github.com/franklinbaldo/ies/commit/17d12410cc4e5b2d926296c698172da9d964ef19) com `main@7ebb836`, preservando os 28 commits novos da base e apenas o teste `tests/aies_unidade_cli/colunas.rs` da PR. [CI rápido](https://github.com/franklinbaldo/ies/actions/runs/42927561183) PASS e [CI oficial](https://github.com/franklinbaldo/ies/actions/runs/42927624598) **7/7 PASS** no SHA exato; PR Ready e aberta, aguardando 🐘/🦡. A `main` recebeu depois a memória OKF 🦡 `22cced4` (sem sobreposição), deixando `behind=1`; [handoff na #565](https://github.com/franklinbaldo/ies/issues/565#issuecomment-6782755925).

**Evidência separada:** BDD-565-02 tem CLI real ↔ HTTPS local ↔ anti-IES/SQLite v38 **PASS sintético**, 4 GETs e 0 POST. O card PEN1 foi inspecionado previamente com hashes íntegros, mas não inclui `procedimento_configurar_detalhe`: confronto semântico bilateral **NOT_RUN**; treinamento/live **NOT_RUN**.

**Próximo encaixe depois do merge:** BDD-565-03 já especificado pelo 🐘. A rota `crates/anti-ies/src/nivel_de_detalhe.rs` ainda responde 405 a POST. Fazer uma transição de colunas tipada no SQLite com uma submissão derivada do formulário, um POST apenas e releitura independente na segunda CLI. O BDD-565-04 depois cobre efeito não comprovado, saída 1 sem retry conforme #681. Não iniciar PR concorrente com #711.

**Cafezinho:** li a [conversa sobre pastas parciais e identidade do PDF](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6782543070): uma lista não completa não prova ausência. É um bom lembrete metodológico, não evidência de colunas.
