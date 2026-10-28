---
type: CommandContract
title: Listar processos da unidade
operation: unidade.listar_processos
command: ies unidade listar-processos
effect: read
implementation: stub
source_commit: 3d4a38c3b5987e4a8fc7f97ded188c4876ed77cd
---

# Listar processos da unidade

`ies unidade listar-processos [--saida ARQ]` consulta a caixa da unidade
ativa. O `--unidade SIGLA` global seleciona o contexto de unidade.

## Entradas

- `--saida ARQ`: destino opcional de exportação dos dados.
- `--json`: formatação estruturada global.
- `--unidade SIGLA`: contexto explícito, sem inventar identificador interno.

## Saída prevista

Modelo `ProcessosDaUnidade`, preservando identidade dos processos,
metadados disponíveis, situação e, quando houver paginação, a distinção
entre coleção completa e parcial/indeterminada.

Não inferir completude apenas porque uma página não mostra botão de avanço.
Um total declarado diferente do total materializado exige aviso.

## Falhas e evidências

A sintaxe é aceita, mas a operação ainda responde `nao_implementado`
com saída 1. Nenhuma requisição ao SEI é feita pelo stub.

[unidade.listar_processos](../evidencias/unidade-listar_processos.md)
e [tabelas e paginação](../../docs/como-o-sei-funciona/tabelas-e-paginacao.md)
são fontes da futura implementação.

A enumeração e o resultado deverão ser confirmados em cassettes por instalação.
