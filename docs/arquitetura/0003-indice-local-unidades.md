---
type: Guide
title: Tabela pública de unidades no índice local
description: Contrato SQL de leitura dos snapshots de unidades do SEI
---

# Tabela pública de unidades (esquema 3, issue #244)

O banco continua privado no filesystem (diretório `0700`, arquivo
`0600`). **Público** significa que agentes com acesso legítimo a esse
arquivo podem executar `SELECT` sem usar a CLI; não significa publicar
o arquivo na rede ou alterar suas permissões.

`lista_unidade(instalacao TEXT, unidade_ativa TEXT, lida_em REAL,
total INTEGER)` tem chave primária `(instalacao, unidade_ativa)`.
`unidade(instalacao TEXT, unidade_ativa TEXT, id TEXT, sigla TEXT,
descricao TEXT, orgao TEXT, lida_em REAL)` tem chave primária
`(instalacao, unidade_ativa, id)`. `instalacao` é a origem
normalizada do SEI; `unidade_ativa` é seu identificador opaco.

```sql
SELECT id, sigla, descricao, orgao, lida_em
FROM unidade
WHERE instalacao = :instalacao AND unidade_ativa = :unidade_ativa
ORDER BY sigla, id;
```

A consulta à `lista_unidade` distingue ainda snapshot ausente e
snapshot válido com zero unidades. Mudança no esquema público requer
migração `PRAGMA user_version`, atualização deste contrato e revisão
dos leitores diretos. **INSERT/UPDATE/DELETE por agentes externos não
são suportados**. A interface pública é só leitura.

A CLI de descoberta do caminho (`ies indice caminho`) e a leitura
Web ainda não foram integradas na primeira fatia da #244.
