---
type: SEIObservation
title: Ambiguidade na reconciliação da criação de processos
description: RED sintético, classificação de dois candidatos e validação live pendente da issue 245
operation: processo.criar
date: 2026-11-06
status: not-run
---

# Reconciliação de candidatos ambíguos — issue #245

## Evidência e método

A estrutura da caixa Gerados, do link observado da própria linha e das
propriedades independentes foi documentada anteriormente no
[relatório de reconciliação](2026-11-05-processo-criar-reconciliacao.md),
na decisão da operação e nos testes da issue #245. Esta fatia não extrapola
o cenário sintético para outras instalações.

O teste novo precedeu a mudança de produção.
A primeira compilação [CI 42182686032](https://github.com/franklinbaldo/ies/actions/runs/42182686032)
falhou pela tentativa de ler um campo de diagnóstico ainda não existente
no RED; o teste foi corrigido sem implementação.
O RED executável [CI 42182787186](https://github.com/franklinbaldo/ies/actions/runs/42182787186)
demonstrou **1 teste novo FAIL, 1.679 PASS, 18 ignored preexistentes**:
o fluxo atribuía `efeito_incerto_na_criacao` a dois candidatos
compatíveis independentemente conferidos. O outro novo teste passou:
duas leituras, uma só correspondência, permitem a classificação positiva
somente depois de conferir o segundo candidato.

## Orçamento sintético

| Situação | GET inicial | POST criador | GET Controle posterior | GET de candidatos | Total |
| --- | ---: | ---: | ---: | ---: | ---: |
| Dois candidatos compatíveis | 3 | 1 | 1 | 6 | **11** |
| Dois candidatos, só um compatível | 3 | 1 | 1 | 6 | **11** |

Todos os dados de prova (origem, hash, cookie, NUP e respostas) são
sintéticos nos testes. O POST é injetado e falha sem resposta; não há
requisição real nem retry. A saída ambígua é 3 e distingue os dois NUPs
verificados. A ausência de correspondência não atesta não aplicação.

## Validação ao vivo em treinamento

- Prévia: **NOT_RUN** (0 requisições).
- Um único `--confirmar`: **NOT_RUN** (0 requisições, 0 POST).
- Verificação por leitura: **NOT_RUN** (0 requisições).
- Escritas e resíduos novos: **0**.
- Motivo: nenhum ensaio de segunda instalação foi iniciado nesta fatia;
  a execução segura depende de acesso autenticado de treinamento e da
  sequência prévia → confirmação única → leitura, sem repetir nenhum POST.

O resíduo histórico da **UnB de treinamento**
`23106.001875/2026-93` (ID 51731) permanece como documentado no
relatório original; **não repetir sua criação**.

A issue #245 continua WIP. Falta provar não aplicação com evidência
independente, completar as saídas públicas previstas na issue e validar
ao vivo em uma segunda instalação. Não declarar a capacidade completa.
