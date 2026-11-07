---
type: SEIObservation
title: Orçamento do ensaio manual de criação com três leituras verificadoras
description: Correção test-first do orçamento da confirmação pós-POST e estado live da issue 245
operation: processo.criar
date: 2026-11-07
status: not-run
---

# 🦉 #245 — orçamento de confirmação no ensaio manual

## Observação antes do RED

No fluxo já implementado em `src/processo_criar/fluxo.rs`, uma
criação com `302 Location` legítimo faz **um POST criador** e só publica
`confirmado: true` após três **GET independentes** em
`src/processo_criar/verificacao_apos_post.rs`: tela do processo,
árvore e propriedades (tipo, especificação, nível e interessados).
O comando de prévia executa a mesma preparação sem criar processo.

O roteiro de ensaio `scripts/ensaio_processo_criar.py` continuava
exigindo `confirmacao = previa + 3` (um POST e duas leituras).
Isso permitia declarar `PASS` com um orçamento insuficiente e
rejeitava uma confirmação corretamente instrumentada com
`ORCAMENTO_DIVERGENTE`. A correção é **`previa + 4`**,
sem novas requisições no IES e sem tocar na lógica de escrita.

## RED e correção

- Testes antes do código: commit `148df3d`; [CI RED
  42338507504](https://github.com/franklinbaldo/ies/actions/runs/42338507504).
  No job `replay processo criar`, **19 testes Python, 3 FAIL**:
  confirmação válida com sete chamadas, rejeição do orçamento antigo
  de seis chamadas e segundo ensaio com trava.
- Código: commit `941d8b6`, alterando somente a constante verificável
  do orçamento para `+4` e explicando o POST + 3 GET.
  A fixture positiva agora registra **3 requisições de prévia, 7 de
  confirmação e 2 da releitura externa**; a negativa exige recusa
  para 3 + 3. Nenhum teste foi ignorado ou enfraquecido.
- Esta mudança corrige o **instrumento de ensaio**; não inventa
  sucesso no SEI nem resolve sozinha a pendência live.

## Validação nesta execução

| Etapa | Estado | Requisições reais ao SEI |
| --- | --- | ---: |
| Testes Python injetados, sem rede | PASS no teste dedicado da execução GREEN | 0 |
| Prévia em segunda instalação de treinamento | NOT_RUN | 0 |
| Um único `--confirmar` | NOT_RUN | 0 |
| Verificação independente por NUP | NOT_RUN | 0 |
| Escrita/resíduo novo | NÃO EXECUTADA / nenhum novo | 0 |

O sandbox não resolve DNS para GitHub/SEI e não dispõe de Cargo. O CI
é a fonte de compilação e testes; a passagem completa dos gates deve
ser consultada no SHA final da PR. A criação histórica da UnB
`23106.001875/2026-93` não foi repetida.

**Restante:** executar com segurança em *outra* instalação de
TREINAMENTO a sequência prévia ao vivo → exatamente um
`--confirmar` → leitura independente e registro do NUP/resíduo
e contagens reais. Se o ciclo não puder ser concluído com segurança,
não escrever. A issue permanece WIP.
