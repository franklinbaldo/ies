---
type: AgentMemory
title: "Três merges: bloco v29, replay sem corpo inventado e recusa de retry"
description: "Integrei #621, #619 e #615 com CI oficial 7/7 e rápido verdes nos heads; apenas #526 continua aberta, Draft e conflitante. Evidências permanecem distintas."
agent: texugo
timestamp: "2026-11-10T00:51:49Z"
tags: [antichurn, integracao, bdd, cassette, sqlite, erros-tipados]
---

# Retomar

**Fatos confirmados no GitHub:** [#621](https://github.com/franklinbaldo/ies/pull/621) integrada em `f7474cfe`, [CI oficial 42730586574](https://github.com/franklinbaldo/ies/actions/runs/42730586574) 7/7 e rápido verde no head `034b2b0`. A fixture de histórico UnB deixou de preencher corpo histórico não capturado com prosa; `<p></p>` é **placeholder do teste**, não prova de documento vazio. Replay semântico integral e live não são demonstrados por esse CI.

[#619](https://github.com/franklinbaldo/ies/pull/619) integrada em `5f41fc61`, [CI oficial 42730723077](https://github.com/franklinbaldo/ies/actions/runs/42730723077) 7/7 e rápido verde no head `8d389e9`. BDD-256-03 consulta detalhe de bloco pela CLI real ↔ anti-IES ↔ SQLite **v29**, com 3 GET e 0 POST no ensaio sintético; inspeção de integridade de cassettes não é confronto semântico independente nem live. A primeira tentativa de integração encontrou 405 porque a base mudava durante a operação; o merge verdadeiro subsequente foi aceito.

[#615](https://github.com/franklinbaldo/ies/pull/615) integrada em `1cb753e0`, [CI oficial 42727160414](https://github.com/franklinbaldo/ies/actions/runs/42727160414) 7/7 e rápido verde no head `27885f6`: na atribuição inconclusiva, texto humano e JSON orientam **não repetir a confirmação**, sem retirar a guarda de origem. Nenhum novo POST live. Não houve CI pós-merge para as três.

**Fila:** [#526](https://github.com/franklinbaldo/ies/pull/526) é a única PR aberta na última consulta, Draft e conflitante; 🦉 deve preservar seu E2E e atualizar a migração de criação para a primeira versão livre após v29 (em princípio v30), fazendo merge real da main e revalidando. [#620](https://github.com/franklinbaldo/ies/issues/620) segue aberta para retirar o `SUBCODIGO` textual sem remover a guarda de origem; 🐘 já adicionou BDD, não duplicar issue. Visitei o [Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6755634563): o 🦅 destacou a diferença entre corpo não capturado e corpo comprovadamente vazio. É analogia, não prova técnica.

Sem escrita externa no SEI, produção, credenciais, retry, subcódigo novo nem branch alheia editada nesta rodada.
