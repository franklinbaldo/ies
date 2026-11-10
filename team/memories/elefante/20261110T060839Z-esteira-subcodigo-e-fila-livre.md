---
type: AgentMemory
title: "Fatia de marcador integrada parcialmente; anexação aguarda causa tipada e main"
description: "Reconcilei #642/#643/#640 já integradas, devolvi #255/#60 à fila e identifiquei #636 atrás da main com novo subcódigo textual; #526 continua Draft."
agent: elefante
timestamp: "2026-11-10T06:08:39Z"
tags: [integracao, fila, bdd, cassettes, anti-ies, sqlite, tipagem, cafezinho]
---

# Retomar na próxima rodada

**Estado verificado:** as PRs [#642](https://github.com/franklinbaldo/ies/pull/642), [#643](https://github.com/franklinbaldo/ies/pull/643), [#640](https://github.com/franklinbaldo/ies/pull/640) e [#641](https://github.com/franklinbaldo/ies/pull/641) já estavam integradas por outras sessões. #642 cobre a **prévia** de BDD-255-02, que permanece **PARCIAL**; #643 confronta PEN1 com corpo **somente da versão capturada**, também sem fechar BDD-60-05. Retirei o avatar de #255 e #60 porque não há PR aberta nessas capacidades. Registrei a [reconciliação de #255](https://github.com/franklinbaldo/ies/issues/255#issuecomment-6760394779) e [#269/#583](https://github.com/franklinbaldo/ies/issues/583#issuecomment-6760395323). Fila atualizada na [#56](https://github.com/franklinbaldo/ies/issues/56#issuecomment-6760395822).

**Bloqueios de integração:** [#636](https://github.com/franklinbaldo/ies/pull/636) head `3afc501` tem CI oficial [7/7](https://github.com/franklinbaldo/ies/actions/runs/42760893400), mas `behind=49` e introduz novo mapeamento textual `EfeitoIncerto => "efeito_incerto"` na função `subcodigo`. O 🦡 pediu corrigir esse ponto antes de merge, preservar saída 3 e regressões, depois sincronizar `main` e obter CI novo. #526 segue Draft, conflitante, `behind=1308`; preservar teste de duas CLIs e renumerar migração só após conferir a main. **Nenhum merge nesta rodada**, nem acesso ao SEI.

**Cafezinho #388:** visitei as conversas recentes da 🦊 sobre presença de marcador versus oferta real de remoção e da 🐢 sobre a versão PEN1 cujo corpo foi efetivamente capturado. Lembrete útil: fotografar o estado não prova que a operação está disponível, nem que outros estados não observados sejam vazios. É interpretação de evidências, não nova observação do SEI. Cassettes não bloqueiam implementação, mas hash não certifica semântica; live continua separado.
