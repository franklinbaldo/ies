---
type: Cassette
title: documento-numero-e-busca-go
installation: go
source_zip: https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-go.zip
archive_member: documento-numero-e-busca-go.json
archive_sha1: 42a08cd3714bb863d136ed98e517221510a31982
cassette_sha256_lf: f26b4140acb8a67c0396479752652d55adcd36ddc2c396f4bd0f1589d234618b
interactions: 15
schema_version: 1
complete: true
---

# documento-numero-e-busca-go

**Instalação:** go (`seitreinamento.go.gov.br`). **Origem:** [ZIP imutável do Internet Archive](https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-go.zip); membro `documento-numero-e-busca-go.json`.

**Verificações:** SHA-1 do ZIP `42a08cd3714bb863d136ed98e517221510a31982`; SHA-256 da cassette após CRLF→LF `f26b4140acb8a67c0396479752652d55adcd36ddc2c396f4bd0f1589d234618b`. Commit de captura: `a7d9f6ecec137e3dcd78c8e770d9f693736a6a2c`.

**Observação:** 15 interações HTTP; métodos GET: 10, POST: 5.

## Operações HTTP registradas

- `protocolo_pesquisar`: 7 requisição(ões)
- `procedimento_trabalhar`: 2 requisição(ões)
- `procedimento_visualizar`: 2 requisição(ões)
- `procedimento_controlar`: 1 requisição(ões)
- `protocolo_pesquisa_rapida`: 1 requisição(ões)
- `documento_imprimir_web`: 1 requisição(ões)
- `assunto_auto_completar_RI1223`: 1 requisição(ões)

## Alcance da evidência

A gravação comprova respostas observadas na instalação indicada. Não demonstra,
por si só, paridade com a CLI IES, corpo de requisições POST ou sucesso sem
conferência pós-escrita. Use os passos relevantes e o contrato antes de implementar.

Consulte [como o SEI funciona](../../docs/como-o-sei-funciona/README.md) e
[o procedimento de replay](../../docs/cassettes.md).
