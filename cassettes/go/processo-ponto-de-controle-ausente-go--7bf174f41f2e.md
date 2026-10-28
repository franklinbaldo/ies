---
type: Cassette
title: processo-ponto-de-controle-ausente-go
installation: go
source_zip: https://archive.org/download/todos-sei-cassettes-2026/lotes/lote-c4bf3e69eabb4b78.zip
archive_member: processo-ponto-de-controle-ausente-go.json
archive_sha1: c4bf3e69eabb4b78b36514bcf9586f300534b0c7
cassette_sha256_lf: 7bf174f41f2ed3875f5f69d18cf49a4d0401beee58d76842b9dfce4d84dbfd78
interactions: 15
schema_version: 1
complete: true
---

# processo-ponto-de-controle-ausente-go

**Instalação:** go (`seitreinamento.go.gov.br`). **Origem:** [ZIP imutável do Internet Archive](https://archive.org/download/todos-sei-cassettes-2026/lotes/lote-c4bf3e69eabb4b78.zip); membro `processo-ponto-de-controle-ausente-go.json`.

**Verificações:** SHA-1 do ZIP `c4bf3e69eabb4b78b36514bcf9586f300534b0c7`; SHA-256 da cassette após CRLF→LF `7bf174f41f2ed3875f5f69d18cf49a4d0401beee58d76842b9dfce4d84dbfd78`. Commit de captura: `54262a4e1b7ce171242e4b5ead35ff8c6c566991`.

**Observação:** 15 interações HTTP; métodos GET: 11, POST: 4.

## Operações HTTP registradas

- `procedimento_escolher_tipo`: 4 requisição(ões)
- `procedimento_visualizar`: 4 requisição(ões)
- `procedimento_gerar`: 3 requisição(ões)
- `procedimento_concluir`: 2 requisição(ões)
- `procedimento_controlar`: 1 requisição(ões)
- `procedimento_trabalhar`: 1 requisição(ões)

## Alcance da evidência

A gravação comprova respostas observadas na instalação indicada. Não demonstra,
por si só, paridade com a CLI IES, corpo de requisições POST ou sucesso sem
conferência pós-escrita. Use os passos relevantes e o contrato antes de implementar.

Consulte [como o SEI funciona](../../docs/como-o-sei-funciona/README.md) e
[o procedimento de replay](../../docs/cassettes.md).
