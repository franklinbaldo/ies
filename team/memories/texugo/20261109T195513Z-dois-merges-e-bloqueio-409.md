---
type: AgentMemory
title: "Duas integrações e 409 de branch atrasada na fila de blocos"
description: "Integrei #599 e #600 com CI 7/7 e rápido verdes; #597 passou 7/7 no head, mas GitHub recusou merge 409 por main avançada; #601 só tem CI rápido."
agent: texugo
timestamp: "2026-11-09T19:55:13Z"
tags: [antichurn, integracao, ci, cassette, sqlite, fila]
---

# Retomar

- Integrei [#599](https://github.com/franklinbaldo/ies/pull/599), merge `52d025a`, confronto **parcial** do link de versões com cassette original UnB; head `037bab25`, CI oficial [#42685160660](https://github.com/franklinbaldo/ies/actions/runs/42685160660) **7/7** e rápido PASS; revisão 🦅 favorável. O único avanço de main desde a sincronização era uma memória OKF.
- Integrei [#600](https://github.com/franklinbaldo/ies/pull/600), merge `4a4fc8e`, preservação de diagnóstico saneado no executor manual, sem novo E2E nem POST; head `0d94cad`, CI oficial [#42683652880](https://github.com/franklinbaldo/ies/actions/runs/42683652880) **7/7** e rápido PASS. A branch estava atrás das memórias e de #599, mas os arquivos eram disjuntos e o REST indicou `mergeable_state=clean`. Nenhum CI pós-merge.
- [#597](https://github.com/franklinbaldo/ies/pull/597) head `a834b2ad` agora tem [CI oficial #42687136391](https://github.com/franklinbaldo/ies/actions/runs/42687136391) **7/7**, rápido PASS, migração v26 em branch, mas tentativa de merge retornou **409 Head branch is out of date** após #599/#600. [Comentei o bloqueio novo](https://github.com/franklinbaldo/ies/pull/597#issuecomment-6749896318): 🐙 deve fazer merge verdadeiro de main e repetir gates sem reiniciar BDD-256-01. **v26 ainda não está integrada**. Na #526, criação continua Draft e conflitante; próxima versão livre deve ser recalculada depois da #597, possivelmente v27.
- [#601](https://github.com/franklinbaldo/ies/pull/601) tem somente CI rápido no head `f6c6acd`, sem CI oficial. [Registrei distinção de provas](https://github.com/franklinbaldo/ies/pull/601#issuecomment-6749916932): PDF realmente escrito e recuperado na UnB de treinamento, mas CLI live retornou saída 3 falsa; correção offline sintética ainda não recebeu uma segunda confirmação live, que exigiria autorização expressa. Não fazer POST para revalidar automaticamente.
- Visitei o [Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388); a conversa da 🦊 sobre código e causa tipada versus bilhete livre ecoa o escopo exato da #600. Analogia, não prova nova.

Sem acesso à produção, credenciais, novos subcódigos ou escrita externa no SEI nesta rodada.
