---
type: Cassette
title: documento-criar-texto-go
installation: go
source_zip: https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-go.zip
archive_member: documento-criar-texto-go.json
archive_sha1: 42a08cd3714bb863d136ed98e517221510a31982
cassette_sha256_lf: 5621c99af5dc84c0625c8ec744fb12e041f3aa306e92232a7bfd47ebbcbba10c
interactions: 21
schema_version: 1
complete: true
---

# documento-criar-texto-go

**Instalação:** go (`seitreinamento.go.gov.br`). **Origem:** [ZIP imutável do Internet Archive](https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-go.zip); membro `documento-criar-texto-go.json`.

**Verificações:** SHA-1 do ZIP `42a08cd3714bb863d136ed98e517221510a31982`; SHA-256 da cassette após CRLF→LF `5621c99af5dc84c0625c8ec744fb12e041f3aa306e92232a7bfd47ebbcbba10c`. Commit de captura: `766546c468c5eddea6e773f5e3568b8010641482`.

**Observação:** 21 interações HTTP; métodos GET: 15, POST: 6.

## Operações HTTP registradas

- `documento_escolher_tipo`: 4 requisição(ões)
- `procedimento_visualizar`: 3 requisição(ões)
- `documento_gerar`: 3 requisição(ões)
- `procedimento_escolher_tipo`: 2 requisição(ões)
- `procedimento_gerar`: 2 requisição(ões)
- `documento_imprimir_web`: 2 requisição(ões)
- `procedimento_controlar`: 1 requisição(ões)
- `procedimento_trabalhar`: 1 requisição(ões)
- `arvore_visualizar`: 1 requisição(ões)
- `editor_montar`: 1 requisição(ões)
- `editor_salvar_conteudo`: 1 requisição(ões)

## Alcance da evidência

A gravação comprova respostas observadas na instalação indicada. Não demonstra,
por si só, paridade com a CLI IES, corpo de requisições POST ou sucesso sem
conferência pós-escrita. Use os passos relevantes e o contrato antes de implementar.

Consulte [como o SEI funciona](../../docs/como-o-sei-funciona/README.md) e
[o procedimento de replay](../../docs/cassettes.md).
