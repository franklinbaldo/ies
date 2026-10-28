---
type: Cassette
title: cadastro-busca-go
installation: go
source_zip: https://archive.org/download/todos-sei-cassettes-2026/lotes/lote-c7a4af8d9d70ddc3.zip
archive_member: cadastro-busca-go.json
archive_sha1: c7a4af8d9d70ddc3599cdd9bccbb98e6d726db78
cassette_sha256_lf: 1685d7572554b8851ffc194521752ce6b59c870d36d8d47b773b86be49a11b4f
interactions: 7
schema_version: 1
complete: true
---

# cadastro-busca-go

**Instalação:** go (`seitreinamento.go.gov.br`). **Origem:** [ZIP imutável do Internet Archive](https://archive.org/download/todos-sei-cassettes-2026/lotes/lote-c7a4af8d9d70ddc3.zip); membro `cadastro-busca-go.json`.

**Verificações:** SHA-1 do ZIP `c7a4af8d9d70ddc3599cdd9bccbb98e6d726db78`; SHA-256 da cassette após CRLF→LF `1685d7572554b8851ffc194521752ce6b59c870d36d8d47b773b86be49a11b4f`. Commit de captura: `7c5464a757b19ef7f2da1a0ceb57a57c99b12914`.

**Observação:** 7 interações HTTP; métodos GET: 4, POST: 3.

## Operações HTTP registradas

- `protocolo_pesquisar`: 3 requisição(ões)
- `contato_auto_completar_pesquisa`: 2 requisição(ões)
- `procedimento_controlar`: 1 requisição(ões)
- `assunto_auto_completar_RI1223`: 1 requisição(ões)

## Alcance da evidência

A gravação comprova respostas observadas na instalação indicada. Não demonstra,
por si só, paridade com a CLI IES, corpo de requisições POST ou sucesso sem
conferência pós-escrita. Use os passos relevantes e o contrato antes de implementar.

Consulte [como o SEI funciona](../../docs/como-o-sei-funciona/README.md) e
[o procedimento de replay](../../docs/cassettes.md).
