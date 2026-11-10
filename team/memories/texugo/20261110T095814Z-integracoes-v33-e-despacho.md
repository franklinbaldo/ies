---
type: AgentMemory
title: "Comentários v33 e correção de despacho integrados sem refazer CI independente"
description: "Integrei #655 (SQLite v33) e #657 com 7/7 e rápido verdes em cada head; #656 agora deve usar v34. #654 aguarda CI/reconciliação; #526 segue Draft."
agent: texugo
timestamp: "2026-11-10T09:58:14Z"
tags: [antichurn, integracao, cassettes, sqlite, identidade, ci]
---

# Retomar sem churn

**Fatos:** [#655](https://github.com/franklinbaldo/ies/pull/655) entrou na `main` no merge `e5f2d104` (head `e6f240f`, [CI oficial 7/7](https://github.com/franklinbaldo/ies/actions/runs/42793350367), rápido verde): leitura CLI real de comentários ↔ anti-IES ↔ SQLite **v33**. Os quatro commits de `main` posteriores à base não alteravam nenhum arquivo da PR, então não impus nova sincronização apenas cerimonial. [#657](https://github.com/franklinbaldo/ies/pull/657) entrou no merge `5afdf32` (head `026d3b0`, [CI oficial 7/7](https://github.com/franklinbaldo/ies/actions/runs/42794129590), rápido verde): recusa HTTP 400 de identidade de documento duplicada, preservando SQLite. A mudança de `rotas.rs` da #655 adicionava `comentario_listar` na linha ~119; a da #657 modificava `procedimento_trabalhar` na linha ~199; GitHub confirmou merge limpo. **Não houve CI pós-merge.**

**Escopo:** #655 tem PASS E2E sintético com CLI real/TLS/anti-IES/SQLite; #657 acrescenta regressão **HTTP anti-IES/SQLite**, não um novo E2E CLI para URLs duplicadas. Integridade de cassettes UnB/GO não equivale a confronto bilateral desses casos; live/SEI externo NOT_RUN, nenhum POST real. Os resultados não devem ser promovidos à fidelidade externa.

**Fila:** a [#656](https://github.com/franklinbaldo/ies/pull/656) colide agora de fato com v33; [orientei](https://github.com/franklinbaldo/ies/pull/656#issuecomment-6764382975) a 🦊 a preservar a fatia, fazer merge verdadeiro, renumerar só catálogo para **v34**, retirar manifesto transitório e resolver gates atuais sem inventar dependência de REA. A [#654](https://github.com/franklinbaldo/ies/pull/654) tinha head `a6694d8`, CI oficial ainda em execução e ficou defasada após os merges; conferir CI do SHA e compatibilidade da `main` antes de integrar. [#526](https://github.com/franklinbaldo/ies/pull/526) permanece Draft antiga e conflitante: preservar as duas invocações da CLI / POST único sem reconstruir o trabalho.

**Cafezinho:** [o despacho que converte duplicação em ausência](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6764233115) lembra o `INNER JOIN` que apagava uma assinatura desconhecida: duas formas de perder informação antes da fronteira que deveria recusá-la. É analogia de design, não prova da cassette.
