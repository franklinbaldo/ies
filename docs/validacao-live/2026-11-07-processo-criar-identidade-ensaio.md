---
type: SEIObservation
title: Identidade observada entre prévia e confirmação do ensaio de criação
description: RED e correção do falso PASS com tipo, especificação ou unidade divergentes no ensaio manual da issue 245
operation: processo.criar
date: 2026-11-07
status: not-run
---

# 🦉 #245 — identidade entre prévia e confirmação no ensaio

## Observação antes do RED

O JSON de `ies processo criar` publica, na prévia e no resultado confirmado,
`tipo`, `especificacao`, `nivel`, `unidade` e `requisicoes`
(`src/processo_criar_cli/apresentacao.rs`).
O roteiro manual `scripts/ensaio_processo_criar.py` conferia tipo, especificação e
nível na prévia, mas não exigia a unidade publicada. Após um POST com resposta
positiva, aceitava `confirmado: true`, NUP e contagem mesmo se **tipo,
especificação, nível ou unidade** fossem diferentes ou faltassem no JSON de
confirmação. O comando Rust já verifica propriedades por leitura; esta mudança
corrige especificamente a **prova independente do roteiro de ensaio**.

## RED antes da implementação

No commit de testes `fcb3508`, foram acrescentados quatro cenários:
prévia sem unidade, confirmação com outro tipo, confirmação sem
especificação e confirmação em outra unidade. O job
[replay processo criar, CI 42342863531](https://github.com/franklinbaldo/ies/actions/runs/42342863531)
executou **23 testes Python, 4 FAIL novos**. Nos quatro casos o roteiro
antigo retornava indevidamente `PASS` (código 0).

## Solução e limites

- A prévia sem unidade observada não permite confirmar (nenhum POST).
- Com POST já iniciado, divergência de qualquer um dos quatro campos
  impede `PASS` e resulta em `CONFIRMACAO_DIVERGENTE`, saída 3.
  O NUP informado permanece no registro privado e a releitura externa
  por NUP **não é pulada**, para conservar o possível resíduo.
- Nenhum retry, POST ou GET adicional é introduzido; continua um
  `--confirmar` por registro de execução e o orçamento esperado permanece
  `3 GET` na prévia, `3 GET + 1 POST + 3 GET` na confirmação do
  cenário sintético padrão e `2 GET` na leitura externa.
- Os campos divergentes não são despejados no relatório público:
  registrar apenas o estado e o NUP, quando observado, impede que um
  resultado bruto da CLI exponha URLs assinadas, cookies ou outros dados.

## Validação e pendências

| Etapa | Estado | Requisições reais ao SEI |
| --- | --- | ---: |
| Testes Python de transporte injetado, sem rede | RED observado; GREEN no CI da PR | 0 |
| Prévia autenticada em segunda instalação de treinamento | NOT_RUN | 0 |
| Confirmação única em treinamento | NOT_RUN | 0 |
| Verificação independente real por NUP | NOT_RUN | 0 |
| Escrita/resíduo novo | NÃO EXECUTADA / nenhum novo | 0 |

Não foi acessada produção nem repetida a escrita incerta histórica da UnB
`23106.001875/2026-93`. A #245 continua WIP: falta um ciclo autenticado
seguro em **outra instalação de TREINAMENTO**, com prévia → exatamente um
`--confirmar` → verificação causal independente, registrando contagens
reais de requisições e resíduos.
