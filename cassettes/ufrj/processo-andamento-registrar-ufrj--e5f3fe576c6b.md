---
type: Cassette
title: processo-andamento-registrar-ufrj
installation: ufrj
source_zip: https://archive.org/download/todos-sei-cassettes-2026/lotes/lote-c7a4af8d9d70ddc3.zip
archive_member: processo-andamento-registrar-ufrj.json
archive_sha1: c7a4af8d9d70ddc3599cdd9bccbb98e6d726db78
cassette_sha256_lf: e5f3fe576c6b006903ca2f2d13f1008e6b1f3b324704043a47ed26c6a9fbf670
interactions: 25
schema_version: 1
complete: true
---

# processo-andamento-registrar-ufrj

**Instalação:** ufrj (`teste.sei.tic.ufrj.br`). **Origem:** [ZIP imutável do Internet Archive](https://archive.org/download/todos-sei-cassettes-2026/lotes/lote-c7a4af8d9d70ddc3.zip); membro `processo-andamento-registrar-ufrj.json`.

**Verificações:** SHA-1 do ZIP `c7a4af8d9d70ddc3599cdd9bccbb98e6d726db78`; SHA-256 da cassette após CRLF→LF `e5f3fe576c6b006903ca2f2d13f1008e6b1f3b324704043a47ed26c6a9fbf670`. Commit de captura: `d440f951db4718d70579b7c0d87076c788a903a0`.

**Observação:** 25 interações HTTP; métodos GET: 19, POST: 6.

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
