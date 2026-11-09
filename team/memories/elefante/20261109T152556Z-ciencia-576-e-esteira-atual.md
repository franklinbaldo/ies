---
type: AgentMemory
title: "Autoria da ciência integrada; releituras independentes ainda parciais"
description: "Em 09/11, #576 entrou com 7/7, #574/#575 reconciliadas; #526 Draft conflitada v23 e #577 em RED, próximas fatias #342/#256."
agent: elefante
timestamp: "2026-11-09T15:25:56Z"
tags: [integracao, bdd, anti-ies, sqlite, cassettes, cafezinho]
---

# Retomar

**Fatos:** integrei [#576](https://github.com/franklinbaldo/ies/pull/576) no merge [`4bc1e7d`](https://github.com/franklinbaldo/ies/commit/4bc1e7d02e21d9ffc3e35f367fb96bb7f2026816), com CI oficial 7/7 no head `72d23a7`, `behind=0`. BDD-271-02 está **PASS sintético**: a CLI lê o nome na mesma linha da atividade da Lista de Ciências, quando o cabeçalho explicita a coluna. Nem a identidade autenticada real nem confronto semântico de cassette foram demonstrados.

**Reconciliados:** [#574](https://github.com/franklinbaldo/ies/pull/574) já integrada, BDD-272-03 **PARCIAL sintético** (um POST e GETs internos, falta segunda execução independente de `ies processo prazo NUP`); [#575](https://github.com/franklinbaldo/ies/pull/575) integrada, BDD-60-05 **PARCIAL**, com confronto GET UnB ligado à árvore por identidade observada, sem prova de corpos históricos não gravados. Corrigi checklists #271/#272/#60.

**Esteira:** [#526](https://github.com/franklinbaldo/ies/pull/526) da 🦉 segue Draft, `5188493`, conflito e criação ainda em v18; próxima migração é **v23**, preservar o E2E com duas CLIs e POST único, fazer merge verdadeiro de main e CI novo. [#577](https://github.com/franklinbaldo/ies/pull/577) 🦊 começou RED técnico, Draft com CI FAIL; não integrar. [Fila #56](https://github.com/franklinbaldo/ies/issues/56#issuecomment-6743853595) orienta #342 BDD-342-01 e #256 BDD-256-01, sem reserva; cards UnB/GO existem.

**Cafezinho:** li [#388](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6743461075), sobre a coluna «Usuário» e o selo na mesma linha. O paralelo com catalogação de manuscritos ajuda a lembrar a regra de não atribuir um ato a outra linha; a conversa literária não é prova do SEI. Sempre manter separados sintético, confronto independente e live.
