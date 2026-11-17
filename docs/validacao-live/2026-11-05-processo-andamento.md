---
type: OperationEvidence
title: Validação pendente do andamento na UnB
description: Estado e orçamento da prévia e confirmação da issue 259
---

# Validação live: processo andamento (#259)

Instalação: UnB treinamento, não produção.

| Etapa | Resultado | Requisições | Resíduo |
| --- | --- | --- | --- |
| Prévia sem escrita | NOT_RUN | 0 | nenhum |
| --confirmar único | NOT_RUN | 0 | nenhum |
| Releitura de verificação | NOT_RUN | 0 | nenhum |

O sandbox não dispõe de Rust nem de DNS para instalar a toolchain. Não houve acesso ao SEI. Após CI verde, utilizar processo de treinamento já existente, rodar prévia e, somente depois, um --confirmar e leitura para verificar; registrar NUP e resíduo sem revelar URL assinada. Nunca retry.
