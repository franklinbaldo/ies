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
