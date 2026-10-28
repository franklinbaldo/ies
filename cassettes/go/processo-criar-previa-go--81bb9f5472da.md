---
type: Cassette
title: processo-criar-previa-go
installation: go
source_zip: https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-go.zip
archive_member: processo-criar-previa-go.json
archive_sha1: 42a08cd3714bb863d136ed98e517221510a31982
cassette_sha256_lf: 81bb9f5472da81d8b2fb67faea4b9172a6c6df930cb93fc82ab7f6b760b8992c
interactions: 6
schema_version: 1
complete: true
---

# processo-criar-previa-go

**Instalação:** go (`seitreinamento.go.gov.br`). **Origem:** [ZIP imutável do Internet Archive](https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-go.zip); membro `processo-criar-previa-go.json`.

**Verificações:** SHA-1 do ZIP `42a08cd3714bb863d136ed98e517221510a31982`; SHA-256 da cassette após CRLF→LF `81bb9f5472da81d8b2fb67faea4b9172a6c6df930cb93fc82ab7f6b760b8992c`. Commit de captura: `87b3996827f4b2be1e0e7ea7e3a9b3a26ba6d8a0`.

**Observação:** 6 interações HTTP; métodos GET: 4, POST: 2.

## Operações HTTP registradas

- `procedimento_escolher_tipo`: 4 requisição(ões)
- `procedimento_controlar`: 1 requisição(ões)
- `procedimento_gerar`: 1 requisição(ões)

## Alcance da evidência

A gravação comprova respostas observadas na instalação indicada. Não demonstra,
por si só, paridade com a CLI IES, corpo de requisições POST ou sucesso sem
conferência pós-escrita. Use os passos relevantes e o contrato antes de implementar.

Consulte [como o SEI funciona](../../docs/como-o-sei-funciona/README.md) e
[o procedimento de replay](../../docs/cassettes.md).
