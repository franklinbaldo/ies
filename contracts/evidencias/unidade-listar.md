---
type: OperationEvidence
title: unidade.listar
operation: unidade.listar
source: ies-local-config
source_commit: f6b3d57996c16c9a979fb339b15232ef69a2593b
source_file: src/unidade_pesquisar_cli.rs
writes: false
ies_implemented: true
upstream_steps: 0
unresolved_steps: 0
---

# unidade.listar — fatia local

- [Implementação](../../src/unidade_pesquisar_cli.rs)
- [Testes offline](../../tests/cli_unidade_indice.rs)
- [Contrato](../cli/unidade-listar.md)

A operação **local sobre snapshot já guardado** não executa passos
Web; as chamadas ao SEI são zero. O `source_commit` identifica a
primeira integração desta fatia Rust e não é uma captura de SEI.
A coleta inicial/atualização pela lupa ainda é outra parte
pendente da issue #244, documentada em
[observação](../../docs/como-o-sei-funciona/lupa-de-unidades.md).
Não confundir compilação CI com ensaio live ou replay.
