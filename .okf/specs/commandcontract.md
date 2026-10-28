---
type: Spec
title: CommandContract
description: Contrato individual de comando da CLI do IES
---

# CommandContract

Contrato de um comando público e seus parâmetros, independente do mecanismo HTTP.

- `operation`: nome lógico equivalente à operação de domínio.
- `command`: comando na CLI `ies`.
- `effect`: `read`, `write` ou `unknown`.
- `implementation`: `stub` enquanto nenhuma operação do SEI foi construída.
- `source_commit`: commit da referência Todos consultada.

O contrato registra sintaxe, saída pretendida, erros e links para
`OperationEvidence`. A matriz do Todos não atesta execução no IES.
Nenhum contrato implementa login, rede ou escrita por si só.
