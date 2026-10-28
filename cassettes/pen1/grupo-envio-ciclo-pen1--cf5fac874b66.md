---
type: Cassette
title: grupo-envio-ciclo-pen1
installation: pen1
source_zip: https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-pen1.zip
archive_member: grupo-envio-ciclo-pen1.json
archive_sha1: 70841f418106b058c4f81c9b13a1322c51ce8cd0
cassette_sha256_lf: cf5fac874b663ddb89a772e8e630e4c8d3f72ea39d94cf8650fdbf37553e9640
interactions: 35
schema_version: 1
complete: true
---

# grupo-envio-ciclo-pen1

**Instalação:** pen1 (`sei.orgao1.tramita.processoeletronico.gov.br`). **Origem:** [ZIP imutável do Internet Archive](https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-pen1.zip); membro `grupo-envio-ciclo-pen1.json`.

**Verificações:** SHA-1 do ZIP `70841f418106b058c4f81c9b13a1322c51ce8cd0`; SHA-256 da cassette após CRLF→LF `cf5fac874b663ddb89a772e8e630e4c8d3f72ea39d94cf8650fdbf37553e9640`. Commit de captura: `3435dcf285dcbeacee3cda9c71bfb80bf654b75c`.

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
