---
type: Cassette
title: documento-incluir-go
installation: go
source_zip: https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-go.zip
archive_member: documento-incluir-go.json
archive_sha1: 42a08cd3714bb863d136ed98e517221510a31982
cassette_sha256_lf: fa4845e16ba97c525208e60179b97c2451ecd11b6e46a3cef7849527d7588f02
interactions: 42
schema_version: 1
complete: true
---

# documento-incluir-go

**Instalação:** go (`seitreinamento.go.gov.br`). **Origem:** [ZIP imutável do Internet Archive](https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-go.zip); membro `documento-incluir-go.json`.

**Verificações:** SHA-1 do ZIP `42a08cd3714bb863d136ed98e517221510a31982`; SHA-256 da cassette após CRLF→LF `fa4845e16ba97c525208e60179b97c2451ecd11b6e46a3cef7849527d7588f02`. Commit de captura: `b0da656830957f05fe6baaac9630c9264799906e`.

**Observação:** 42 interações HTTP; métodos GET: 29, POST: 13.

## Operações HTTP registradas

- `procedimento_visualizar`: 11 requisição(ões)
- `procedimento_escolher_tipo`: 6 requisição(ões)
- `procedimento_gerar`: 5 requisição(ões)
- `procedimento_concluir`: 4 requisição(ões)
- `procedimento_controlar`: 3 requisição(ões)
- `documento_gerar_multiplo`: 3 requisição(ões)
- `procedimento_trabalhar`: 2 requisição(ões)
- `documento_escolher_tipo`: 2 requisição(ões)
- `documento_gerar`: 2 requisição(ões)
- `arvore_visualizar`: 1 requisição(ões)
- `editor_montar`: 1 requisição(ões)
- `editor_salvar_conteudo`: 1 requisição(ões)
- `documento_imprimir_web`: 1 requisição(ões)

## Alcance da evidência

A gravação comprova respostas observadas na instalação indicada. Não demonstra,
por si só, paridade com a CLI IES, corpo de requisições POST ou sucesso sem
conferência pós-escrita. Use os passos relevantes e o contrato antes de implementar.

Consulte [como o SEI funciona](../../docs/como-o-sei-funciona/README.md) e
[o procedimento de replay](../../docs/cassettes.md).
