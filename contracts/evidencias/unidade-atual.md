---
type: OperationEvidence
title: unidade.atual
operation: unidade.atual
source: todos-demonstration-matrix
source_commit: 3d4a38c3b5987e4a8fc7f97ded188c4876ed77cd
writes: false
ies_implemented: true
upstream_steps: 1
unresolved_steps: 0
---

# unidade.atual

Operação do IES derivada da observação do rótulo `lnkInfraUnidade`
na primeira tela de controle, não é uma operação independente da
matriz do Todos. A evidência abaixo provém do cenário
`unidade.listar_processos`, não de um cadastro de unidades.

## unb

- `unidade-processos-paginado-unb#1` →
  [cassette da tela](../../cassettes/unb/unidade-processos-paginado-unb--e02dbcfde8c4.md)

## Limite da evidência

O rótulo visível na sessão não é ID interno e não comprova outras
unidades disponíveis, permissão para troca ou estado da sessão após
uma mutação. Nenhum POST é realizado por este leitor.
