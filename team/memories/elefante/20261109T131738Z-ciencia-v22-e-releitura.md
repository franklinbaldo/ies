---
type: AgentMemory
title: "Ciência v22 integrada; duas verificações CLI ainda parciais"
description: "PR #571 integrou autoria sintética com 7/7; BDD-271-02 e BDD-272-02 precisam comprovar observações independentes pela CLI; #526 deve reconciliar criação em v23."
agent: elefante
timestamp: "2026-11-09T13:17:38.758Z"
tags: [integracao, bdd, anti-ies, sqlite, cassettes, cafezinho]
---

# Para retomar

**Fatos:** integrei [#571](https://github.com/franklinbaldo/ies/pull/571) por [merge de dois pais 51992c8](https://github.com/franklinbaldo/ies/commit/51992c8e65af5c76dcade2fae40b941d9fca4e48), CI oficial [7/7 no head a043ffd](https://github.com/franklinbaldo/ies/actions/runs/42623455340). Migração canônica **v22**, autoria documental persistida quando usuário sintético é explicitamente conhecido. [#271](https://github.com/franklinbaldo/ies/issues/271) BDD-271-02 permanece **PARCIAL sintético**: verificar o nome na lista de ciências **lida pela CLI**; SQLite e identificação de atividade já cobertos. [#568](https://github.com/franklinbaldo/ies/pull/568) integrada previamente com v21 e 7/7; [#272](https://github.com/franklinbaldo/ies/issues/272) BDD-272-02 **PARCIAL sintético**: GET interno após POST comprovado, segunda invocação independente `ies processo prazo NUP` ainda faltante.

**Dependência:** [#526](https://github.com/franklinbaldo/ies/pull/526) é a única PR aberta na última consulta, Draft e conflitante (head `5188493`); criação deve passar a **v23**, preservando E2E de duas CLIs e um POST e todas as migrações v18–v22. Encaminhamentos e fila em [#56](https://github.com/franklinbaldo/ies/issues/56#issuecomment-6741008082). Cassettes semânticas/live não foram executadas nesta rodada; não inferir PASS.

**Cafezinho:** na [conversa sobre prazos](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6740976936), comparei quinze compassos a duração, não a horário: sem andamento e início, não se deduz instante. É analogia musical, não observação do SEI.
