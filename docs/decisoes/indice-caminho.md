---
type: ImplementationDecision
title: Expor o caminho configurado do índice em comando próprio
operation: indice.caminho
implementation: partial
status: adopted-for-scope
scope: descoberta de caminho SQLite sem criar arquivos ou acessar o SEI
selected_strategy: subcomando-local-indice
alternatives: subcomando-local-indice,acrescentar-status
evidence_level: configuracao-local-e-testes-sinteticos
live_validation: not-applicable
---

# Índice: descobrir caminho sem sessão

## Alternativas

- **subcomando-local-indice** (escolhido): `ies indice caminho` resolve
  as opções do índice, sem autenticação ou GET.
- **acrescentar-status**: misturaria a descoberta do banco com o
  diagnóstico da sessão existente e ampliaria seu JSON.

## Decisão e consequências

Manter a funcionalidade de descoberta separada em um subcomando.
O IES consulta somente a configuração local, inclusive as regras
`IES_CONFIG_FILE`, `IES_INDICE_FILE` e `IES_SEM_INDICE`.
O resultado é um caminho **configurado**, nunca uma prova de que o
banco existe, contém todas as unidades ou pode ser aberto.

Os testes verificam a superfície clap e o cálculo independente
do banco. O contrato de leitura das tabelas permanece em
[unidades SQLite](../arquitetura/0003-indice-local-unidades.md).
A obtenção integral da lista via lupa, replay e ensaio live
ainda são pendentes da #244.

[Contrato](../../contracts/cli/indice-caminho.md) ·
[Código](../../src/indice_caminho_cli.rs)
