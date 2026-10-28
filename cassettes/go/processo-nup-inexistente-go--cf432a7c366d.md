---
type: Cassette
title: processo-nup-inexistente-go
installation: go
source_zip: https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-go.zip
archive_member: processo-nup-inexistente-go.json
archive_sha1: 42a08cd3714bb863d136ed98e517221510a31982
cassette_sha256_lf: cf432a7c366d7369d5a7e3bbfae9e995866ea7aafff61609606e1b166563df03
interactions: 8
schema_version: 1
complete: true
---

# processo-nup-inexistente-go

**Instalação:** go (`seitreinamento.go.gov.br`). **Origem:** [ZIP imutável do Internet Archive](https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-go.zip); membro `processo-nup-inexistente-go.json`.

**Verificações:** SHA-1 do ZIP `42a08cd3714bb863d136ed98e517221510a31982`; SHA-256 da cassette após CRLF→LF `cf432a7c366d7369d5a7e3bbfae9e995866ea7aafff61609606e1b166563df03`. Commit de captura: `bd10c5cf967314566442317aed26354d08ed9f19`.

**Observação:** 8 interações HTTP; métodos GET: 6, POST: 2.

## Operações HTTP registradas

- `protocolo_pesquisa_rapida`: 2 requisição(ões)
- `procedimento_controlar`: 1 requisição(ões)
- `procedimento_trabalhar`: 1 requisição(ões)
- `procedimento_visualizar`: 1 requisição(ões)
- `procedimento_alterar`: 1 requisição(ões)
- `anotacao_registrar`: 1 requisição(ões)
- `procedimento_consultar_historico`: 1 requisição(ões)

## Alcance da evidência

A gravação comprova respostas observadas na instalação indicada. Não demonstra,
por si só, paridade com a CLI IES, corpo de requisições POST ou sucesso sem
conferência pós-escrita. Use os passos relevantes e o contrato antes de implementar.

Consulte [como o SEI funciona](../../docs/como-o-sei-funciona/README.md) e
[o procedimento de replay](../../docs/cassettes.md).
