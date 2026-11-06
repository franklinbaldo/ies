---
type: SEIObservation
title: Releitura conservadora dos Gerados após falha do POST criador
description: Requisições sintéticas e pendências de confirmação da issue 245
operation: processo.criar
date: 2026-11-06
status: not_run
---

# Criação de processo — releitura pós-POST (#245)

A tela e a operação foram observadas anteriormente no Todos, nas
capturas do SEI e na reconciliação independente do ensaio da UnB.
O [ensaio histórico da UnB](2026-11-05-processo-criar-reconciliacao.md)
criou `23106.001875/2026-93` (ID 51731), confirmado por leitura do
🐘 Elefante. O POST histórico **não foi repetido**.

## Transporte sintético desta fatia

| Cenário | Esperado | Orçamento |
| --- | --- | --- |
| POST retorna falha de rede | Uma releitura fresca dos Gerados, saída 3 | 3 GET + 1 POST + 1 GET = 5 |
| POST e nova leitura retornam falha | Preservar ambas causas, saída 3 | 3 GET + 1 POST + 1 GET = 5 |
| Classificador sem candidatos | Ausência apenas observada, não autoriza retry | 0 (offline) |
| Classificador com NUP, ID, tipo e especificação exatos | Uma ou várias correspondências, conforme provas | 0 (offline) |

O resultado no transporte ainda não executa as leituras independentes
de tipo/especificação dos candidatos. Por isso não conclui aplicada nem
não aplicada, e preserva saída 3. A separação protege contra falso
negativo, concorrência na caixa Gerados e duplicação de escrita.

## Live da sessão

| Etapa | Resultado | Requisições reais | Escritas |
| --- | --- | ---: | ---: |
| Prévia autenticada em treinamento | NOT_RUN | 0 | 0 |
| Confirmar uma nova criação em treinamento | NOT_RUN | 0 | 0 |
| Reconciliação automática com quatro desfechos | NOT_RUN — incompleta | 0 | 0 |
| Leitura independente pós-POST | NOT_RUN | 0 | 0 |

**Total desta fatia: 0 requisições reais, 0 POST real, 0 resíduo novo.**
Não extrapolar resultados de transporte sintético para SEI real.

## RED e CI

Testes de classificação foram publicados com shim primeiro, no
head `4bb1b5a` ([run 42162251428](https://github.com/franklinbaldo/ies/actions/runs/42162251428)).
O workflow foi cancelado após novos commits; **contagem isolada RED
não aferida**, salvo eventual log de teste disponível. Os testes
posteriores também foram publicados antes do novo ramo de releitura;
a contagem RED isolada igualmente não foi aferida.
GREEN: conferir o CI do head final; não aproveitar o de SHA anterior.

## Ainda falta

Ligar a leitura real de `processo NUP` para cada candidato, conferir
tipo/especificação e unidade, implementar os quatro desfechos com
causas e saídas, orçamento integral e ensaio live autorizado em
treinamento (prévia → um único `--confirmar` → verificação independente
e anotação de resíduo). O único `--confirmar` histórico da UnB não deve
ser reexecutado.
