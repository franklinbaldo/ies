---
type: Cassette
title: unidade-listagens-go
installation: go
source_zip: https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-go.zip
archive_member: unidade-listagens-go.json
archive_sha1: 42a08cd3714bb863d136ed98e517221510a31982
cassette_sha256_lf: 2ea07a6d26ed04fe47466992bd9f0c08746023eb9fe3384da18d875a78a9d71e
interactions: 167
schema_version: 1
complete: true
---

# unidade-listagens-go

**Instalação:** go (`seitreinamento.go.gov.br`). **Origem:** [ZIP imutável do Internet Archive](https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-go.zip); membro `unidade-listagens-go.json`.

**Verificações:** SHA-1 do ZIP `42a08cd3714bb863d136ed98e517221510a31982`; SHA-256 da cassette após CRLF→LF `2ea07a6d26ed04fe47466992bd9f0c08746023eb9fe3384da18d875a78a9d71e`. Commit de captura: `a09d72e6310faa927566795f891e558357b62693`.

**Observação:** 167 interações HTTP; métodos GET: 7, POST: 160.

## Operações HTTP registradas

- `bloco_assinatura_listar`: 145 requisição(ões)
- `acompanhamento_listar`: 18 requisição(ões)
- `procedimento_controlar`: 1 requisição(ões)
- `controle_prazo_listar`: 1 requisição(ões)
- `procedimento_sobrestado_listar`: 1 requisição(ões)
- `texto_padrao_interno_listar`: 1 requisição(ões)

## Alcance da evidência

A gravação comprova respostas observadas na instalação indicada. Não demonstra,
por si só, paridade com a CLI IES, corpo de requisições POST ou sucesso sem
conferência pós-escrita. Use os passos relevantes e o contrato antes de implementar.

Consulte [como o SEI funciona](../../docs/como-o-sei-funciona/README.md) e
[o procedimento de replay](../../docs/cassettes.md).
