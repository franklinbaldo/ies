---
type: Cassette
title: bloco-obter-concluido-go
installation: go
source_zip: https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-go.zip
archive_member: bloco-obter-concluido-go.json
archive_sha1: 42a08cd3714bb863d136ed98e517221510a31982
cassette_sha256_lf: a721708d382b7cc081894135a2a3b4d84cb8ce64502e066657f98190139a8a13
interactions: 64
schema_version: 1
complete: true
---

# bloco-obter-concluido-go

**Instalação:** go (`seitreinamento.go.gov.br`). **Origem:** [ZIP imutável do Internet Archive](https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-go.zip); membro `bloco-obter-concluido-go.json`.

**Verificações:** SHA-1 do ZIP `42a08cd3714bb863d136ed98e517221510a31982`; SHA-256 da cassette após CRLF→LF `a721708d382b7cc081894135a2a3b4d84cb8ce64502e066657f98190139a8a13`. Commit de captura: `1d21a432322149293e4e020c2f9b88c4e6754329`.

**Observação:** 64 interações HTTP; métodos GET: 4, POST: 60.

## Operações HTTP registradas

- `bloco_assinatura_listar`: 62 requisição(ões)
- `procedimento_controlar`: 1 requisição(ões)
- `rel_bloco_protocolo_listar`: 1 requisição(ões)

## Alcance da evidência

A gravação comprova respostas observadas na instalação indicada. Não demonstra,
por si só, paridade com a CLI IES, corpo de requisições POST ou sucesso sem
conferência pós-escrita. Use os passos relevantes e o contrato antes de implementar.

Consulte [como o SEI funciona](../../docs/como-o-sei-funciona/README.md) e
[o procedimento de replay](../../docs/cassettes.md).
