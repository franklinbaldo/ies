---
type: AgentMemory
title: "Andamento v27 integrado, duplicata de paginação encerrada e limite de origem preservado"
description: "Merges #610/#612/#602 (CI 7/7 nos heads), #608 fechada como duplicata da #611, e #613 sinalizada por remover também a barreira contra origem não autorizada."
agent: texugo
timestamp: "2026-11-09T22:57:32.382Z"
tags: [antichurn, integracao, sqlite, cassettes, seguranca, bdd]
---

# Retomar

Fatos: [#610](https://github.com/franklinbaldo/ies/pull/610) merge `4967bf7` (corrige documentação de atribuição inconclusiva); [#612](https://github.com/franklinbaldo/ies/pull/612) merge `47149bb` (confronto **parcial** com metadados originais de versões UnB); [#602](https://github.com/franklinbaldo/ies/pull/602) merge `eb68c32` (prévia BDD-259-01 por CLI real → anti-IES → SQLite, v27 `0027_andamento_processo.sql`). Nos três heads, CI oficial 7/7 e rápido success, mas no #602 **só na tentativa 2** após falha intermitente preexistente em teste de índice, acompanhada na [#609](https://github.com/franklinbaldo/ies/issues/609): reexecução aprovada não corrige o flake. Não houve CI pós-merge.

A #602 contém PASS **sintético**, inspeção de integridade UnB/GO 50 interações, confronto semântico independente e live NOT_RUN. #612 é confronto **parcial** de cassette, não corpus histórico integral. #610 é retificação documental de live de atribuição com resultado causal inconclusivo, não nova tentativa.

A [#608](https://github.com/franklinbaldo/ies/pull/608) foi fechada como duplicata da [#611](https://github.com/franklinbaldo/ies/pull/611), sem excluir branch/commits. #611 é via única de BDD-256-02, deve preservar E2E, incorporar main por merge verdadeiro, renumerar **só paginação** para v28 se livre, corrigir Clippy e assert de versão, e repetir CI 7/7. [Comentário](https://github.com/franklinbaldo/ies/pull/611#issuecomment-6753697050). #526 segue Draft, criação processual com migração muito antiga: calcular versão efetiva na hora, sem reiniciar E2E.

Na [#613](https://github.com/franklinbaldo/ies/pull/613#issuecomment-6753708510), retirei falso dilema: **remover a chave de ensaio** pedida pelo proprietário não requer **remover a checagem de origem**; o patch atual autoriza escrita para qualquer sessão, contrariando limite só treino/homologação. Pedi preservar a fronteira antes do primeiro request, sem novo subcódigo/gate. CI verde não prova cumprimento desse limite.

[No Cafezinho](https://github.com/franklinbaldo/ies/issues/388), a conversa dos colegas sobre arquivo, edição e autoria do efeito reforça a diferença entre evidência de estado e prova causal; analogia literária, não observação SEI. Nenhuma operação externa ou produção nesta rodada.
