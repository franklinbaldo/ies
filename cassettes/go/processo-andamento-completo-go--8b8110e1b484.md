---
type: Cassette
title: processo-andamento-completo-go
installation: go
source_zip: https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-go.zip
archive_member: processo-andamento-completo-go.json
archive_sha1: 42a08cd3714bb863d136ed98e517221510a31982
cassette_sha256_lf: 8b8110e1b484bed896b9acab847ff80375e10437ac203dd5bdb3c3f6d1c40a3b
interactions: 12
schema_version: 1
complete: true
---

# processo-andamento-completo-go

**Instalação:** go (`seitreinamento.go.gov.br`). **Origem:** [ZIP imutável do Internet Archive](https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-go.zip); membro `processo-andamento-completo-go.json`.

**Verificações:** SHA-1 do ZIP `42a08cd3714bb863d136ed98e517221510a31982`; SHA-256 da cassette após CRLF→LF `8b8110e1b484bed896b9acab847ff80375e10437ac203dd5bdb3c3f6d1c40a3b`. Commit de captura: `61f884d7cc75ab2aeb9e7ef6bc79e1ccf738bde7`.

**Observação:** 12 interações HTTP; métodos GET: 10, POST: 2.

## Operações HTTP registradas

- `procedimento_consultar_historico`: 3 requisição(ões)
- `procedimento_visualizar`: 2 requisição(ões)
- `procedimento_alterar`: 2 requisição(ões)
- `anotacao_registrar`: 2 requisição(ões)
- `procedimento_controlar`: 1 requisição(ões)
- `protocolo_pesquisa_rapida`: 1 requisição(ões)
- `procedimento_trabalhar`: 1 requisição(ões)

## Alcance da evidência

A gravação comprova respostas observadas na instalação indicada. Não demonstra,
por si só, paridade com a CLI IES, corpo de requisições POST ou sucesso sem
conferência pós-escrita. Use os passos relevantes e o contrato antes de implementar.

Consulte [como o SEI funciona](../../docs/como-o-sei-funciona/README.md) e
[o procedimento de replay](../../docs/cassettes.md).
