---
type: Cassette
title: processo-prazo-go
installation: go
source_zip: https://archive.org/download/todos-sei-cassettes-2026/lotes/lote-c7a4af8d9d70ddc3.zip
archive_member: processo-prazo-go.json
archive_sha1: c7a4af8d9d70ddc3599cdd9bccbb98e6d726db78
cassette_sha256_lf: 4d9b8462dae912c136275a290ade6614495046f4e683d07f5ccfe64c1cbd9162
interactions: 33
schema_version: 1
complete: true
---

# processo-prazo-go

**Instalação:** go (`seitreinamento.go.gov.br`). **Origem:** [ZIP imutável do Internet Archive](https://archive.org/download/todos-sei-cassettes-2026/lotes/lote-c7a4af8d9d70ddc3.zip); membro `processo-prazo-go.json`.

**Verificações:** SHA-1 do ZIP `c7a4af8d9d70ddc3599cdd9bccbb98e6d726db78`; SHA-256 da cassette após CRLF→LF `4d9b8462dae912c136275a290ade6614495046f4e683d07f5ccfe64c1cbd9162`. Commit de captura: `1df32b274cb7a9e23568b5100e68d251a5b81dbd`.

**Observação:** 33 interações HTTP; métodos GET: 26, POST: 7.

## Operações HTTP registradas

- `procedimento_visualizar`: 11 requisição(ões)
- `controle_prazo_definir`: 11 requisição(ões)
- `procedimento_escolher_tipo`: 4 requisição(ões)
- `procedimento_gerar`: 3 requisição(ões)
- `procedimento_concluir`: 2 requisição(ões)
- `procedimento_controlar`: 1 requisição(ões)
- `procedimento_trabalhar`: 1 requisição(ões)

## Alcance da evidência

A gravação comprova respostas observadas na instalação indicada. Não demonstra,
por si só, paridade com a CLI IES, corpo de requisições POST ou sucesso sem
conferência pós-escrita. Use os passos relevantes e o contrato antes de implementar.

Consulte [como o SEI funciona](../../docs/como-o-sei-funciona/README.md) e
[o procedimento de replay](../../docs/cassettes.md).
