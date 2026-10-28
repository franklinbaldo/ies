---
type: Cassette
title: bloco-lista-obter-buscar-go
installation: go
source_zip: https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-go.zip
archive_member: bloco-lista-obter-buscar-go.json
archive_sha1: 42a08cd3714bb863d136ed98e517221510a31982
cassette_sha256_lf: 1c804abc3d1ebea673b18ecb913859f5dbeafb701cac3c9539057c0f9c5ee4f7
interactions: 299
schema_version: 1
complete: true
---

# bloco-lista-obter-buscar-go

**Instalação:** go (`seitreinamento.go.gov.br`). **Origem:** [ZIP imutável do Internet Archive](https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-go.zip); membro `bloco-lista-obter-buscar-go.json`.

**Verificações:** SHA-1 do ZIP `42a08cd3714bb863d136ed98e517221510a31982`; SHA-256 da cassette após CRLF→LF `1c804abc3d1ebea673b18ecb913859f5dbeafb701cac3c9539057c0f9c5ee4f7`. Commit de captura: `4010be231ef3ae66e2c756789f4516f45e3db732`.

**Observação:** 299 interações HTTP; métodos GET: 9, POST: 290.

## Operações HTTP registradas

- `bloco_assinatura_listar`: 297 requisição(ões)
- `procedimento_controlar`: 1 requisição(ões)
- `rel_bloco_protocolo_listar`: 1 requisição(ões)

## Alcance da evidência

A gravação comprova respostas observadas na instalação indicada. Não demonstra,
por si só, paridade com a CLI IES, corpo de requisições POST ou sucesso sem
conferência pós-escrita. Use os passos relevantes e o contrato antes de implementar.

Consulte [como o SEI funciona](../../docs/como-o-sei-funciona/README.md) e
[o procedimento de replay](../../docs/cassettes.md).
