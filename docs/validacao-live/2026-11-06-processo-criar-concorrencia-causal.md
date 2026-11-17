---
type: SEIObservation
title: Criação de processo e concorrência indistinguível após POST sem resposta
description: RED sintético e efeito causal inconclusivo em conta compartilhada; ensaio live pendente
operation: processo.criar
date: 2026-11-06
status: not-run
---

# #245 — Reconciliação causal em conta compartilhada

## Observação anterior ao RED

A caixa Gerados, o `id_procedimento`, o NUP, a árvore e os campos
de propriedades foram previamente observados na UnB treinamento e
conferidos nos replays de 15 cassettes da #245. Outra sessão da mesma
conta pode criar um processo com todos os campos coincidentes durante
o intervalo entre as fotografias anterior e posterior ao POST sem resposta.
A observação dessas telas não contém identificador causal da requisição.

## RED sintético aferido

[CI 42280203578](https://github.com/franklinbaldo/ies/actions/runs/42280203578),
commit test-first `35badd17`: o teste
`red_candidato_identico_de_outro_agente_nao_prova_autoria_do_post`
falhou porque o IES antigo devolvia `Criacao { confirmado: true, nup:
Some(...), causa_post: Some("falha_de_rede"), requisicoes: 8 }`.
**1 FAIL novo, 1.734 PASS, 22 ignored preexistentes**. Rustfmt
também sinalizou a formatação do teste inicial; a falha funcional
é distinta e está documentada no log.

## Mudança e orçamento

O classificador distingue coincidência única de atribuição causal; a
escrita agora permanece **inconclusiva (saída 3)** mesmo após verificar
ID/NUP, tipo, especificação, nível e interessados de um candidato.
Retorna `candidato_unico_sem_prova_causal`, possível resíduo, NUP
candidato e causa original. Para um candidato: **8 pedidos sintéticos**
(3 GET prévios, 1 POST criador tentado, 1 GET Gerados posterior e 3 GET
verificadores). Dois candidatos com só um coincidente: **11 pedidos
sintéticos**. Exatamente **1 POST criador**, sem retry, redirect
automático, segredo em log ou execução de JavaScript.

## Treinamento — NOT_RUN

| Etapa | Estado | Requisições reais |
| --- | --- | --- |
| Prévia autenticada na segunda instalação | NOT_RUN | 0 |
| Uma confirmação autorizada de criação | NOT_RUN | 0 |
| Releitura verificadora independente | NOT_RUN | 0 |

**Resíduo novo nesta fatia: nenhum (sem escritas reais).**
A criação histórica na UnB treinamento `23106.001875/2026-93`
(`id_procedimento` 51731) já foi reconciliada; **NÃO repetir seu POST**.
A capacidade ainda requer prévia válida em segunda instalação de
treinamento → um único `--confirmar` → leitura independente e registro
do novo resíduo, apenas se for seguro completar a sequência.
