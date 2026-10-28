---
type: Cassette
title: documento-escritas-go
installation: go
source_zip: https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-go.zip
archive_member: documento-escritas-go.json
archive_sha1: 42a08cd3714bb863d136ed98e517221510a31982
cassette_sha256_lf: 16f83179ecaabecb1de24598a8aeda51088954c116f998514cfa8cc3a8045100
interactions: 69
schema_version: 1
complete: true
---

# documento-escritas-go

**Instalação:** go (`seitreinamento.go.gov.br`). **Origem:** [ZIP imutável do Internet Archive](https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-go.zip); membro `documento-escritas-go.json`.

**Verificações:** SHA-1 do ZIP `42a08cd3714bb863d136ed98e517221510a31982`; SHA-256 da cassette após CRLF→LF `16f83179ecaabecb1de24598a8aeda51088954c116f998514cfa8cc3a8045100`. Commit de captura: `d35772d8a8ec6390d4669138c4f17584eb290254`.

**Observação:** 69 interações HTTP; métodos GET: 51, POST: 18.

## Operações HTTP registradas

- `procedimento_visualizar`: 15 requisição(ões)
- `documento_escolher_tipo`: 8 requisição(ões)
- `arvore_visualizar`: 7 requisição(ões)
- `documento_imprimir_web`: 5 requisição(ões)
- `documento_receber`: 5 requisição(ões)
- `documento_alterar`: 4 requisição(ões)
- `documento_gerar`: 3 requisição(ões)
- `editor_montar`: 3 requisição(ões)
- `documento_verificar_assinatura`: 3 requisição(ões)
- `documento_assinar`: 3 requisição(ões)
- `documento_recebido_duplicado`: 3 requisição(ões)
- `procedimento_escolher_tipo`: 2 requisição(ões)
- `procedimento_gerar`: 2 requisição(ões)
- `editor_salvar_conteudo`: 2 requisição(ões)
- `procedimento_controlar`: 1 requisição(ões)
- `procedimento_trabalhar`: 1 requisição(ões)
- `documento_upload_anexo`: 1 requisição(ões)
- `documento_excluir`: 1 requisição(ões)

## Alcance da evidência

A gravação comprova respostas observadas na instalação indicada. Não demonstra,
por si só, paridade com a CLI IES, corpo de requisições POST ou sucesso sem
conferência pós-escrita. Use os passos relevantes e o contrato antes de implementar.

Consulte [como o SEI funciona](../../docs/como-o-sei-funciona/README.md) e
[o procedimento de replay](../../docs/cassettes.md).
