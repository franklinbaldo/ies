---
type: Cassette
title: processo-arquivo-pdf-go
installation: go
source_zip: https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-go.zip
archive_member: processo-arquivo-pdf-go.json
archive_sha1: 42a08cd3714bb863d136ed98e517221510a31982
cassette_sha256_lf: 05c4342a73ec491391d3e62f43b7f50ead135a4137689322a91243118e661db3
interactions: 7
schema_version: 1
complete: true
---

# processo-arquivo-pdf-go

**Instalação:** go (`seitreinamento.go.gov.br`). **Origem:** [ZIP imutável do Internet Archive](https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-go.zip); membro `processo-arquivo-pdf-go.json`.

**Verificações:** SHA-1 do ZIP `42a08cd3714bb863d136ed98e517221510a31982`; SHA-256 da cassette após CRLF→LF `05c4342a73ec491391d3e62f43b7f50ead135a4137689322a91243118e661db3`. Commit de captura: `8901a363ae4864f42bfc597a97d8cff03af545ec`.

**Observação:** 7 interações HTTP; métodos GET: 5, POST: 2.

## Operações HTTP registradas

- `procedimento_gerar_pdf`: 2 requisição(ões)
- `procedimento_controlar`: 1 requisição(ões)
- `protocolo_pesquisa_rapida`: 1 requisição(ões)
- `procedimento_trabalhar`: 1 requisição(ões)
- `procedimento_visualizar`: 1 requisição(ões)
- `exibir_arquivo`: 1 requisição(ões)

## Alcance da evidência

A gravação comprova respostas observadas na instalação indicada. Não demonstra,
por si só, paridade com a CLI IES, corpo de requisições POST ou sucesso sem
conferência pós-escrita. Use os passos relevantes e o contrato antes de implementar.

Consulte [como o SEI funciona](../../docs/como-o-sei-funciona/README.md) e
[o procedimento de replay](../../docs/cassettes.md).
