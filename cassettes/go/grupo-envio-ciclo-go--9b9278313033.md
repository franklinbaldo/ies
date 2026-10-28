---
type: Cassette
title: grupo-envio-ciclo-go
installation: go
source_zip: https://archive.org/download/todos-sei-cassettes-2026/lotes/lote-c7a4af8d9d70ddc3.zip
archive_member: grupo-envio-ciclo-go.json
archive_sha1: c7a4af8d9d70ddc3599cdd9bccbb98e6d726db78
cassette_sha256_lf: 9b92783130336be00fc61bb7de0000dabf94e26e1432a5a2625cbbe58a2adc24
interactions: 34
schema_version: 1
complete: true
---

# grupo-envio-ciclo-go

**Instalação:** go (`seitreinamento.go.gov.br`). **Origem:** [ZIP imutável do Internet Archive](https://archive.org/download/todos-sei-cassettes-2026/lotes/lote-c7a4af8d9d70ddc3.zip); membro `grupo-envio-ciclo-go.json`.

**Verificações:** SHA-1 do ZIP `c7a4af8d9d70ddc3599cdd9bccbb98e6d726db78`; SHA-256 da cassette após CRLF→LF `9b92783130336be00fc61bb7de0000dabf94e26e1432a5a2625cbbe58a2adc24`. Commit de captura: `e7ccc14edd1462d8f57d7209681a0be6d66de2ed`.

**Observação:** 34 interações HTTP; métodos GET: 25, POST: 9.

## Operações HTTP registradas

- `grupo_unidade_listar`: 15 requisição(ões)
- `unidade_auto_completar_envio_processo`: 6 requisição(ões)
- `grupo_unidade_alterar`: 5 requisição(ões)
- `grupo_unidade_cadastrar`: 3 requisição(ões)
- `grupo_unidade_consultar`: 3 requisição(ões)
- `procedimento_controlar`: 1 requisição(ões)
- `grupo_unidade_excluir`: 1 requisição(ões)

## Alcance da evidência

A gravação comprova respostas observadas na instalação indicada. Não demonstra,
por si só, paridade com a CLI IES, corpo de requisições POST ou sucesso sem
conferência pós-escrita. Use os passos relevantes e o contrato antes de implementar.

Consulte [como o SEI funciona](../../docs/como-o-sei-funciona/README.md) e
[o procedimento de replay](../../docs/cassettes.md).
