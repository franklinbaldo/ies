---
type: Cassette
title: cadastro-listas-go
installation: go
source_zip: https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-go.zip
archive_member: cadastro-listas-go.json
archive_sha1: 42a08cd3714bb863d136ed98e517221510a31982
cassette_sha256_lf: beb707305040ab2f1aa120c6b833dc90fae08063ba8817a99c20b46122bafb5f
interactions: 19
schema_version: 1
complete: true
---

# cadastro-listas-go

**Instalação:** go (`seitreinamento.go.gov.br`). **Origem:** [ZIP imutável do Internet Archive](https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-go.zip); membro `cadastro-listas-go.json`.

**Verificações:** SHA-1 do ZIP `42a08cd3714bb863d136ed98e517221510a31982`; SHA-256 da cassette após CRLF→LF `beb707305040ab2f1aa120c6b833dc90fae08063ba8817a99c20b46122bafb5f`. Commit de captura: `3efbe147aecc13af0417a26acb6d6b9e26216e68`.

**Observação:** 19 interações HTTP; métodos GET: 12, POST: 7.

## Operações HTTP registradas

- `documento_escolher_tipo`: 6 requisição(ões)
- `procedimento_escolher_tipo`: 4 requisição(ões)
- `documento_receber`: 2 requisição(ões)
- `procedimento_controlar`: 1 requisição(ões)
- `procedimento_gerar`: 1 requisição(ões)
- `hipotese_legal_select_nome_base_legal`: 1 requisição(ões)
- `protocolo_pesquisar`: 1 requisição(ões)
- `protocolo_pesquisa_rapida`: 1 requisição(ões)
- `procedimento_trabalhar`: 1 requisição(ões)
- `procedimento_visualizar`: 1 requisição(ões)

## Alcance da evidência

A gravação comprova respostas observadas na instalação indicada. Não demonstra,
por si só, paridade com a CLI IES, corpo de requisições POST ou sucesso sem
conferência pós-escrita. Use os passos relevantes e o contrato antes de implementar.

Consulte [como o SEI funciona](../../docs/como-o-sei-funciona/README.md) e
[o procedimento de replay](../../docs/cassettes.md).
