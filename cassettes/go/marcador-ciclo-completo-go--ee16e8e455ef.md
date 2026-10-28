---
type: Cassette
title: marcador-ciclo-completo-go
installation: go
source_zip: https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-go.zip
archive_member: marcador-ciclo-completo-go.json
archive_sha1: 42a08cd3714bb863d136ed98e517221510a31982
cassette_sha256_lf: ee16e8e455efcef54cecc59b1c57379b79105adcbef4120b84c1bb532387b545
interactions: 52
schema_version: 1
complete: true
---

# marcador-ciclo-completo-go

**Instalação:** go (`seitreinamento.go.gov.br`). **Origem:** [ZIP imutável do Internet Archive](https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-go.zip); membro `marcador-ciclo-completo-go.json`.

**Verificações:** SHA-1 do ZIP `42a08cd3714bb863d136ed98e517221510a31982`; SHA-256 da cassette após CRLF→LF `ee16e8e455efcef54cecc59b1c57379b79105adcbef4120b84c1bb532387b545`. Commit de captura: `5148257044bff438ec0ff19e76dcbd9c7747be48`.

**Observação:** 52 interações HTTP; métodos GET: 34, POST: 18.

## Operações HTTP registradas

- `marcador_listar`: 40 requisição(ões)
- `marcador_cadastrar`: 4 requisição(ões)
- `marcador_alterar`: 3 requisição(ões)
- `marcador_desativar`: 2 requisição(ões)
- `procedimento_controlar`: 1 requisição(ões)
- `marcador_reativar`: 1 requisição(ões)
- `marcador_excluir`: 1 requisição(ões)

## Alcance da evidência

A gravação comprova respostas observadas na instalação indicada. Não demonstra,
por si só, paridade com a CLI IES, corpo de requisições POST ou sucesso sem
conferência pós-escrita. Use os passos relevantes e o contrato antes de implementar.

Consulte [como o SEI funciona](../../docs/como-o-sei-funciona/README.md) e
[o procedimento de replay](../../docs/cassettes.md).
