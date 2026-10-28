---
type: Cassette
title: pesquisa-paginas-go
installation: go
source_zip: https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-go.zip
archive_member: pesquisa-paginas-go.json
archive_sha1: 42a08cd3714bb863d136ed98e517221510a31982
cassette_sha256_lf: c4018448aa0444390e15dcbc3a21272ff05401d9f3a44145329d945f2dd88f70
interactions: 9
schema_version: 1
complete: true
---

# pesquisa-paginas-go

**Instalação:** go (`seitreinamento.go.gov.br`). **Origem:** [ZIP imutável do Internet Archive](https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-go.zip); membro `pesquisa-paginas-go.json`.

**Verificações:** SHA-1 do ZIP `42a08cd3714bb863d136ed98e517221510a31982`; SHA-256 da cassette após CRLF→LF `c4018448aa0444390e15dcbc3a21272ff05401d9f3a44145329d945f2dd88f70`. Commit de captura: `eb3a1b868ec72687da69ddd4fc9011974bf0bddc`.

**Observação:** 9 interações HTTP; métodos GET: 3, POST: 6.

## Operações HTTP registradas

- `protocolo_pesquisar`: 7 requisição(ões)
- `procedimento_controlar`: 1 requisição(ões)
- `assunto_auto_completar_RI1223`: 1 requisição(ões)

## Alcance da evidência

A gravação comprova respostas observadas na instalação indicada. Não demonstra,
por si só, paridade com a CLI IES, corpo de requisições POST ou sucesso sem
conferência pós-escrita. Use os passos relevantes e o contrato antes de implementar.

Consulte [como o SEI funciona](../../docs/como-o-sei-funciona/README.md) e
[o procedimento de replay](../../docs/cassettes.md).
