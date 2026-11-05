---
type: OperationEvidence
title: indice.caminho
operation: indice.caminho
source: ies-local-config
source_commit: 758c9f7b40519dc24dddcb60e80d80d9e0ffdc37
source_file: src/indice_caminho_cli.rs
writes: false
ies_implemented: true
upstream_steps: 0
unresolved_steps: 0
---

# indice.caminho

Não depende do oráculo Todos ou de uma interface do SEI.
É um comando de infraestrutura puramente local.

- [Implementação](../../src/indice_caminho_cli.rs)
- [Resolução de caminhos](../../src/indice_local.rs)
- [Contrato](../cli/indice-caminho.md)

O caminho é apenas a localização configurada; não prova
existência, integridade, versão, credenciais nem disponibilidade
de lista completa de unidades. Nenhum request é realizado.
