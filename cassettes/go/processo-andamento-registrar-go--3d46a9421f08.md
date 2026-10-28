---
type: Cassette
title: processo-andamento-registrar-go
installation: go
source_zip: https://archive.org/download/todos-sei-cassettes-2026/lotes/lote-c7a4af8d9d70ddc3.zip
archive_member: processo-andamento-registrar-go.json
archive_sha1: c7a4af8d9d70ddc3599cdd9bccbb98e6d726db78
cassette_sha256_lf: 3d46a9421f085c07a88c78ae543e3258dfc46ab6241bc306858933a83a0777b0
interactions: 27
schema_version: 1
complete: true
---

# processo-andamento-registrar-go

**Instalação:** go (`seitreinamento.go.gov.br`). **Origem:** [ZIP imutável do Internet Archive](https://archive.org/download/todos-sei-cassettes-2026/lotes/lote-c7a4af8d9d70ddc3.zip); membro `processo-andamento-registrar-go.json`.

**Verificações:** SHA-1 do ZIP `c7a4af8d9d70ddc3599cdd9bccbb98e6d726db78`; SHA-256 da cassette após CRLF→LF `3d46a9421f085c07a88c78ae543e3258dfc46ab6241bc306858933a83a0777b0`. Commit de captura: `dd84607d1e2d6afc214a3df8dbfad79b1734ffe4`.

**Observação:** 27 interações HTTP; métodos GET: 20, POST: 7.

## Operações HTTP registradas

- `procedimento_visualizar`: 6 requisição(ões)
- `procedimento_atualizar_andamento`: 5 requisição(ões)
- `procedimento_escolher_tipo`: 4 requisição(ões)
- `procedimento_gerar`: 3 requisição(ões)
- `procedimento_consultar_historico`: 3 requisição(ões)
- `procedimento_concluir`: 2 requisição(ões)
- `procedimento_controlar`: 1 requisição(ões)
- `procedimento_trabalhar`: 1 requisição(ões)

## Alcance da evidência

A gravação comprova respostas observadas na instalação indicada. Não demonstra,
por si só, paridade com a CLI IES, corpo de requisições POST ou sucesso sem
conferência pós-escrita. Use os passos relevantes e o contrato antes de implementar.

Consulte [como o SEI funciona](../../docs/como-o-sei-funciona/README.md) e
[o procedimento de replay](../../docs/cassettes.md).
