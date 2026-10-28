---
type: Cassette
title: processo-andamento-completo-pen2
installation: pen2
source_zip: https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-pen2.zip
archive_member: processo-andamento-completo-pen2.json
archive_sha1: 32cd3f7feb5a4a80b3b363e9750b9ce0b593d2ff
cassette_sha256_lf: cc1197c6243433f51c365d0a8d2eb954f3b31ec39b9f178f545661abf6648d55
interactions: 16
schema_version: 1
complete: true
---

# processo-andamento-completo-pen2

**Instalação:** pen2 (`sei.orgao2.tramita.processoeletronico.gov.br`). **Origem:** [ZIP imutável do Internet Archive](https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-pen2.zip); membro `processo-andamento-completo-pen2.json`.

**Verificações:** SHA-1 do ZIP `32cd3f7feb5a4a80b3b363e9750b9ce0b593d2ff`; SHA-256 da cassette após CRLF→LF `cc1197c6243433f51c365d0a8d2eb954f3b31ec39b9f178f545661abf6648d55`. Commit de captura: `142fc97a2058875171f98f8a59ecefc3a3ab57bc`.

**Observação:** 16 interações HTTP; métodos GET: 14, POST: 2.

## Operações HTTP registradas

- `procedimento_consultar_historico`: 3 requisição(ões)
- `procedimento_visualizar`: 2 requisição(ões)
- `procedimento_alterar`: 2 requisição(ões)
- `anotacao_registrar`: 2 requisição(ões)
- `procedimento_atribuicao_cadastrar`: 2 requisição(ões)
- `procedimento_relacionar`: 2 requisição(ões)
- `procedimento_controlar`: 1 requisição(ões)
- `protocolo_pesquisa_rapida`: 1 requisição(ões)
- `procedimento_trabalhar`: 1 requisição(ões)

## Alcance da evidência

A gravação comprova respostas observadas na instalação indicada. Não demonstra,
por si só, paridade com a CLI IES, corpo de requisições POST ou sucesso sem
conferência pós-escrita. Use os passos relevantes e o contrato antes de implementar.

Consulte [como o SEI funciona](../../docs/como-o-sei-funciona/README.md) e
[o procedimento de replay](../../docs/cassettes.md).
