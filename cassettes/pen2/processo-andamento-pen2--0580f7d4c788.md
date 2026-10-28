---
type: Cassette
title: processo-andamento-pen2
installation: pen2
source_zip: https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-pen2.zip
archive_member: processo-andamento-pen2.json
archive_sha1: 32cd3f7feb5a4a80b3b363e9750b9ce0b593d2ff
cassette_sha256_lf: 0580f7d4c7889848f00ec30a069ba4a3a746fae0d68ee89e9e0dcb65f938b2cb
interactions: 11
schema_version: 1
complete: true
---

# processo-andamento-pen2

**Instalação:** pen2 (`sei.orgao2.tramita.processoeletronico.gov.br`). **Origem:** [ZIP imutável do Internet Archive](https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-pen2.zip); membro `processo-andamento-pen2.json`.

**Verificações:** SHA-1 do ZIP `32cd3f7feb5a4a80b3b363e9750b9ce0b593d2ff`; SHA-256 da cassette após CRLF→LF `0580f7d4c7889848f00ec30a069ba4a3a746fae0d68ee89e9e0dcb65f938b2cb`. Commit de captura: `1802f19a52b2b36455de93e9e6c60d1c6877421d`.

**Observação:** 11 interações HTTP; métodos GET: 10, POST: 1.

## Operações HTTP registradas

- `procedimento_controlar`: 1 requisição(ões)
- `protocolo_pesquisa_rapida`: 1 requisição(ões)
- `procedimento_trabalhar`: 1 requisição(ões)
- `procedimento_visualizar`: 1 requisição(ões)
- `andamento_marcador_gerenciar`: 1 requisição(ões)
- `andamento_marcador_cadastrar`: 1 requisição(ões)
- `procedimento_alterar`: 1 requisição(ões)
- `anotacao_registrar`: 1 requisição(ões)
- `procedimento_atribuicao_cadastrar`: 1 requisição(ões)
- `procedimento_relacionar`: 1 requisição(ões)
- `procedimento_consultar_historico`: 1 requisição(ões)

## Alcance da evidência

A gravação comprova respostas observadas na instalação indicada. Não demonstra,
por si só, paridade com a CLI IES, corpo de requisições POST ou sucesso sem
conferência pós-escrita. Use os passos relevantes e o contrato antes de implementar.

Consulte [como o SEI funciona](../../docs/como-o-sei-funciona/README.md) e
[o procedimento de replay](../../docs/cassettes.md).
