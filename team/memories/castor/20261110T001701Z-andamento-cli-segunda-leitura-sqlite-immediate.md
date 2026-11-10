---
type: AgentMemory
title: "Confirmação de andamento com segunda CLI e transação SQLite única"
description: "BDD-259-02 integrado na PR #614, CI 7/7; merge da main preserva a política canônica de treinamento e elimina janela de atualização perdida; próxima fatia BDD-259-03."
agent: castor
timestamp: "2026-11-10T00:17:01Z"
tags: [ies, anti-ies, andamento, bdd, sqlite, merge, ci, cassettes, qualidade]
---

# Próxima rodada

**Fato:** a [PR #614](https://github.com/franklinbaldo/ies/pull/614) do BDD-259-02 entrou na `main` pelo merge `5982321e`. O merge verdadeiro `0e3d97c1` foi montado com dois pais, preservando a política da `main` (`origem_de_treinamento` como guardião único, com treinamento autorizado, nunca produção) e descartando o whitelist concorrente e o manifesto temporário. A CLI real executa um POST no anti-IES, confirma a alteração na primeira saída e uma **segunda CLI** verifica o histórico no SQLite compartilhado. Em `iesei::andamentos::acrescentar`, a leitura e o regravar rodam sob a mesma transação `IMMEDIATE`, evitando a janela de atualização perdida; ainda não há teste concorrente dedicado para afirmar robustez sob carga.

**CI:** [oficial #42726520004](https://github.com/franklinbaldo/ies/actions/runs/42726520004) **7/7 PASS**, [rápido #42726493787](https://github.com/franklinbaldo/ies/actions/runs/42726493787) PASS, head `b461a845`, `behind=0` na última comparação antes da integração. Os primeiros gates capturaram Rustfmt e Clippy (empréstimo mutável desnecessário); ambos corrigidos, sem desligar checks.

**Três níveis:** IES↔anti-IES↔SQLite **PASS sintético**; 23 interações UnB e 27 GO em cards verificados por integridade previamente, mas confronto semântico independente **NOT_RUN**; treinamento **NOT_RUN** nesta fatia. Sem acesso externo, credenciais ou subcódigos novos.

**CONTINUAR:** [issue #259](https://github.com/franklinbaldo/ies/issues/259) segue aberta com **BDD-259-03** de efeito incerto, saída 3 e zero retry. Examinar as PRs abertas antes de criar branch nova, para não repetir a duplicação #616. Não voltar à #614 já integrada.

**Cafezinho:** na [#388](https://github.com/franklinbaldo/ies/issues/388), a observação da 🐢 sobre não combinar corpo de cassette com metadados inventados reforça a distinção entre estado sintético útil e atributo efetivamente observado. É orientação metodológica, não nova prova do SEI.
