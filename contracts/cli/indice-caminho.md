---
type: CommandContract
title: Caminho do índice local SQLite
operation: indice.caminho
command: ies indice caminho
effect: read
implementation: partial
source_commit: 758c9f7b40519dc24dddcb60e80d80d9e0ffdc37
---

# Descobrir o caminho do índice local

`ies indice caminho [--json]` resolve o caminho do banco
SQLite usado pelo IES, sem abrir o banco, consultar o SEI,
validar cookies ou enviar requisições. Não cria diretórios ou
arquivos. O caminho pode ainda não existir.

A fonte de configuração segue `IES_CONFIG_FILE` ou os caminhos
privados individuais, pelas mesmas regras de
[`configuracao`](../../src/configuracao.rs). O local é
`IES_INDICE_FILE`, se não vazio; senão `ies.sqlite3` no
diretório de `IES_URL_CONTROLE_FILE`.
`IES_SEM_INDICE` não vazio e diferente de `0` desliga o índice.

Sem `--json`, mostra o caminho ou informa que o índice está
desligado. Com `--json`, devolve `indice:
"configurado"|"desligado"`, `caminho` (texto ou `null`),
`acesso_remoto: false` e `banco_aberto: false`.
A configuração recusada retorna subcódigo de configuração
existente e código 1. O comando não garante que o arquivo
exista, tenha esquema atual ou esteja utilizável.

A [tabela pública](../../docs/arquitetura/0003-indice-local-unidades.md)
permite `SELECT` por agentes já autorizados a acessar o
arquivo privado. Esse contrato não autoriza gravação externa.
A [decisão](../../docs/decisoes/indice-caminho.md)
registra as alternativas.
