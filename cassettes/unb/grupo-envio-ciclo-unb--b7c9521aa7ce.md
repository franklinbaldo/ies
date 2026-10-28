---
type: Cassette
title: grupo-envio-ciclo-unb
installation: unb
source_zip: https://archive.org/download/todos-sei-cassettes-2026/lotes/lote-c7a4af8d9d70ddc3.zip
archive_member: grupo-envio-ciclo-unb.json
archive_sha1: c7a4af8d9d70ddc3599cdd9bccbb98e6d726db78
cassette_sha256_lf: b7c9521aa7ce67896bc2840a0622dfcb4cd9ecb5943eee95bc3f2fae9fcd01b9
interactions: 35
schema_version: 1
complete: true
---

# grupo-envio-ciclo-unb

**Instalação:** unb (`sei.treinamento.unb.br`). **Origem:** [ZIP imutável do Internet Archive](https://archive.org/download/todos-sei-cassettes-2026/lotes/lote-c7a4af8d9d70ddc3.zip); membro `grupo-envio-ciclo-unb.json`.

**Verificações:** SHA-1 do ZIP `c7a4af8d9d70ddc3599cdd9bccbb98e6d726db78`; SHA-256 da cassette após CRLF→LF `b7c9521aa7ce67896bc2840a0622dfcb4cd9ecb5943eee95bc3f2fae9fcd01b9`. Commit de captura: `67c50d774a3d062cbf448dd2a96c79281b817b4c`.

**Observação:** 35 interações HTTP; métodos GET: 27, POST: 8.

## Operações HTTP registradas

- `grupo_unidade_listar`: 16 requisição(ões)
- `grupo_unidade_alterar`: 6 requisição(ões)
- `unidade_auto_completar_envio_processo`: 4 requisição(ões)
- `grupo_unidade_consultar`: 4 requisição(ões)
- `grupo_unidade_cadastrar`: 3 requisição(ões)
- `procedimento_controlar`: 1 requisição(ões)
- `grupo_unidade_excluir`: 1 requisição(ões)

## Alcance da evidência

A gravação comprova respostas observadas na instalação indicada. Não demonstra,
por si só, paridade com a CLI IES, corpo de requisições POST ou sucesso sem
conferência pós-escrita. Use os passos relevantes e o contrato antes de implementar.

Consulte [como o SEI funciona](../../docs/como-o-sei-funciona/README.md) e
[o procedimento de replay](../../docs/cassettes.md).
