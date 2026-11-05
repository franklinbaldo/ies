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
| Formatação Rust do CI #948 | FAIL (corrigida no branch, nova validação pendente) | 0 |
| Replay integral das cassettes publicadas | NOT_RUN | 0 |
| Prévia ao vivo em UnB/UFRJ | NOT_RUN | 0 |
| Criação ao vivo | NOT_RUN — autorização ausente | 0 |
| Leitura posterior de NUP criado | NOT_RUN | 0 |

Nenhuma requisição foi enviada a qualquer instalação do SEI.
Não houve sessão de treinamento disponível neste ambiente.
A escrita ao vivo depende da autorização literal
“escrita autorizada” na issue #245.

O código da CLI, da prévia e da confirmação foi implementado
na PR #249 e testado em transporte injetado (sem rede real).
Os testes offline NÃO certificam paridade live IES/Todos.
RED antes da implementação: contagem não medida. A revisão
final do CI no head atualizado precisa ser consultada no GitHub.
