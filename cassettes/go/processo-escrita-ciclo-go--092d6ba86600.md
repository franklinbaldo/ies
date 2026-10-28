---
type: Cassette
title: processo-escrita-ciclo-go
installation: go
source_zip: https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-go.zip
archive_member: processo-escrita-ciclo-go.json
archive_sha1: 42a08cd3714bb863d136ed98e517221510a31982
cassette_sha256_lf: 092d6ba866009cdd60860711ef7b5e60e7db5bd9b8223ca4613b72b3ec9161d2
interactions: 59
schema_version: 1
complete: true
---

# processo-escrita-ciclo-go

**Instalação:** go (`seitreinamento.go.gov.br`). **Origem:** [ZIP imutável do Internet Archive](https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-go.zip); membro `processo-escrita-ciclo-go.json`.

**Verificações:** SHA-1 do ZIP `42a08cd3714bb863d136ed98e517221510a31982`; SHA-256 da cassette após CRLF→LF `092d6ba866009cdd60860711ef7b5e60e7db5bd9b8223ca4613b72b3ec9161d2`. Commit de captura: `fdcfe8957232dcce8b34e8579c51f61e9909d006`.

**Observação:** 59 interações HTTP; métodos GET: 41, POST: 18.

## Operações HTTP registradas

- `procedimento_visualizar`: 17 requisição(ões)
- `documento_escolher_tipo`: 8 requisição(ões)
- `documento_receber`: 5 requisição(ões)
- `procedimento_escolher_tipo`: 4 requisição(ões)
- `procedimento_concluir`: 4 requisição(ões)
- `procedimento_gerar`: 3 requisição(ões)
- `documento_gerar`: 3 requisição(ões)
- `arvore_visualizar`: 3 requisição(ões)
- `documento_recebido_duplicado`: 3 requisição(ões)
- `documento_excluir`: 2 requisição(ões)
- `procedimento_controlar`: 1 requisição(ões)
- `procedimento_trabalhar`: 1 requisição(ões)
- `editor_montar`: 1 requisição(ões)
- `editor_salvar_conteudo`: 1 requisição(ões)
- `documento_imprimir_web`: 1 requisição(ões)
- `documento_upload_anexo`: 1 requisição(ões)
- `procedimento_reabrir`: 1 requisição(ões)

## Alcance da evidência

A gravação comprova respostas observadas na instalação indicada. Não demonstra,
por si só, paridade com a CLI IES, corpo de requisições POST ou sucesso sem
conferência pós-escrita. Use os passos relevantes e o contrato antes de implementar.

Consulte [como o SEI funciona](../../docs/como-o-sei-funciona/README.md) e
[o procedimento de replay](../../docs/cassettes.md).
