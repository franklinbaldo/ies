---
type: SEIObservation
title: Estado da validação de iniciar processo
operation: processo.criar
date: 2026-11-05
status: not-run
---

# Iniciar processo — validação da issue #245

| Etapa | Resultado | Requisições IES |
| --- | --- | ---: |
| Inspeção offline do Todos e captura SEI 4.0.9 | OBSERVADO INDIRETAMENTE | 0 |
| RED Rust (falhas antes do código) | NOT_RUN | 0 |
| Testes Rust com transporte sintético (4.0/4.1) | PASS no CI #948 | 0 |
| Compilação, Clippy, suíte Rust e subcódigos | PASS no CI #948 | 0 |
| Gate completo no head 1d263ee | PASS (run 42069555057) | 0 |
| Recusa HTTP do GET preserva status (integração) | PASS no teste sintético | 0 |
| Replay integral das cassettes publicadas | NOT_RUN | 0 |
| Prévia ao vivo em UnB/UFRJ | NOT_RUN | 0 |
| Criação ao vivo | NOT_RUN — autorizada em treinamento (05/11/2026), ainda não executada | 0 |
| Leitura posterior de NUP criado | NOT_RUN | 0 |

Nenhuma requisição foi enviada a qualquer instalação do SEI.
Não houve sessão de treinamento disponível neste ambiente.
A escrita ao vivo em treinamento está autorizada pelo proprietário
(05/11/2026): prévia, um único `--confirmar` e leitura verificadora,
com o resíduo registrado aqui. Ainda não foi executada.

O código da CLI, da prévia e da confirmação foi implementado
na PR #249 e testado em transporte injetado (sem rede real).
Os testes offline NÃO certificam paridade live IES/Todos.
RED antes da implementação: contagem não medida. A revisão
final do CI no head atualizado precisa ser consultada no GitHub.
