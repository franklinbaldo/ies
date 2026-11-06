---
type: SEIObservation
title: Guarda pré-POST do fluxo de criação de processos
description: RED e validação da recusa segura antes de escrever quando Gerados não tem cobertura integral
operation: processo.criar
date: 2026-11-05
status: not_run
---

# Guarda pré-POST — issue #245

Continuação do [relatório de reconciliação](2026-11-05-processo-criar-reconciliacao.md).
A seção foi separada porque o relatório consolidado ultrapassaria 20.000 bytes.

## Continuação 🦉 — guarda pré-POST no Controle (#245)

Teste RED sintético publicado no commit `022f988` antes do código:
[CI 42155921037](https://github.com/franklinbaldo/ies/actions/runs/42155921037),
**1 falha**, 1622 testes aprovados e 17 ignorados preexistentes.
O teste falhou porque o código tentou um POST não previsto no roteiro
de três GET. A guarda posterior impede a escrita quando Gerados está
paginada; a prévia de mesmo formulário continua possível.
GREEN e gates do head final: consultar CI do branch (não inferir
PASS a partir da execução RED).

| Operação | Situação live | Requisições reais | Escritas |
| --- | --- | ---: | ---: |
| Recusa por cobertura insuficiente | NOT_RUN — somente transporte sintético | 0 | 0 |
| Prévia em instalação de treinamento | NOT_RUN | 0 | 0 |
| Confirmação nova em treinamento | NOT_RUN — nenhuma tentativa | 0 | 0 |
| Reconciliação pós-POST na CLI | NOT_RUN — ainda não implementada | 0 | 0 |

**Nesta continuação: 0 requisições reais ao SEI, 0 escritas,
0 resíduos novos.** O NUP histórico da UnB segue
`23106.001875/2026-93` (id 51731); sua confirmação não foi repetida.

## Coruja — resposta inesperada pós-POST (06/11/2026)

Regressão com três testes sintéticos escritos antes da implementação,
RED no [CI 42168412163](https://github.com/franklinbaldo/ies/actions/runs/42168412163):
**3 testes falharam, 1.633 passaram, 17 ignorados preexistentes**. O RED
provou que um HTML 200 ou uma `Location` ambígua encerrava a execução
sem consumir a releitura prevista. A nova fatia exige cinco pedidos exatos
(3 GET iniciais, 1 POST criador, 1 GET fresco do Controle), sem retry e sem
seguir o redirecionamento suspeito, inclusive quando a releitura falha.

O teste e o código são sintéticos: **prévia live NOT_RUN; confirmação live
NOT_RUN; releitura live NOT_RUN; 0 GET reais, 0 POST reais, nenhum resíduo
novo**. Não repetir o POST histórico de treinamento na UnB que gerou
`23106.001875/2026-93` (id 51731).

## Coruja — recusa explícita versus candidato concorrente (06/11/2026)

Observação anterior: comportamento de validação Infra no leitor Web e
resposta HTTP 403 já documentados nos testes existentes; a conta SEI
pode ser compartilhada. RED sintético do commit `e61bc47` no
[CI 42180304698](https://github.com/franklinbaldo/ies/actions/runs/42180304698):
**2 testes FAIL, 1.676 PASS, 18 ignorados preexistentes**.
As duas recusas foram indevidamente classificadas como sucesso ao ver
um candidato concorrente com o mesmo tipo/especificação.
Após a correção, os dois casos exigem **3 GET + 1 POST**, zero GET
pós-recusa, e preservam a causa do SEI (Infra 200 ou HTTP 403).

LIVE: **NOT_RUN**, zero requisições reais, zero POST real, zero escritas,
zero resíduos novos. O NUP histórico da UnB não foi recriado.
