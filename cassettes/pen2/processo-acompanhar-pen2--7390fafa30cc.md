---
type: Cassette
title: processo-acompanhar-pen2
installation: pen2
source_zip: https://archive.org/download/todos-sei-cassettes-2026/lotes/lote-28a245e07bf7d358.zip
archive_member: processo-acompanhar-pen2.json
archive_sha1: 28a245e07bf7d358a322981744dc5e79a835ba94
cassette_sha256_lf: 7390fafa30cc46020991888659f93f6f12417c1a09f651d5a68ea55f0984b3ff
interactions: 47
schema_version: 1
complete: true
---

# processo-acompanhar-pen2

**Instalação:** pen2 (`sei.orgao2.tramita.processoeletronico.gov.br`). **Origem:** [ZIP imutável do Internet Archive](https://archive.org/download/todos-sei-cassettes-2026/lotes/lote-28a245e07bf7d358.zip); membro `processo-acompanhar-pen2.json`.

**Verificações:** SHA-1 do ZIP `28a245e07bf7d358a322981744dc5e79a835ba94`; SHA-256 da cassette após CRLF→LF `7390fafa30cc46020991888659f93f6f12417c1a09f651d5a68ea55f0984b3ff`. Commit de captura: `21c4d54c1ae688548cb8ffc8a37a9471f0b730e7`.

**Observação:** 47 interações HTTP; métodos GET: 38, POST: 9.

## Operações HTTP registradas

- `acompanhamento_gerenciar`: 13 requisição(ões)
- `procedimento_visualizar`: 12 requisição(ões)
- `acompanhamento_cadastrar`: 9 requisição(ões)
- `procedimento_escolher_tipo`: 4 requisição(ões)
- `procedimento_gerar`: 3 requisição(ões)
- `acompanhamento_excluir`: 2 requisição(ões)
- `procedimento_concluir`: 2 requisição(ões)
- `procedimento_controlar`: 1 requisição(ões)
- `procedimento_trabalhar`: 1 requisição(ões)

## Alcance da evidência

A gravação comprova respostas observadas na instalação indicada. Não demonstra,
por si só, paridade com a CLI IES, corpo de requisições POST ou sucesso sem
conferência pós-escrita. Use os passos relevantes e o contrato antes de implementar.

Consulte [como o SEI funciona](../../docs/como-o-sei-funciona/README.md) e
[o procedimento de replay](../../docs/cassettes.md).
