---
type: Cassette
title: processo-acompanhar-pen1
installation: pen1
source_zip: https://archive.org/download/todos-sei-cassettes-2026/lotes/lote-7e4954690958f7a7.zip
archive_member: processo-acompanhar-pen1.json
archive_sha1: 7e4954690958f7a73caf0534d0c05f32e891dde9
cassette_sha256_lf: 4c1e1ee1d144b981bb26488a49a957edc222938fed3ccedfc9d1bb99dbe23c59
interactions: 45
schema_version: 1
complete: true
---

# processo-acompanhar-pen1

**Instalação:** pen1 (`sei.orgao1.tramita.processoeletronico.gov.br`). **Origem:** [ZIP imutável do Internet Archive](https://archive.org/download/todos-sei-cassettes-2026/lotes/lote-7e4954690958f7a7.zip); membro `processo-acompanhar-pen1.json`.

**Verificações:** SHA-1 do ZIP `7e4954690958f7a73caf0534d0c05f32e891dde9`; SHA-256 da cassette após CRLF→LF `4c1e1ee1d144b981bb26488a49a957edc222938fed3ccedfc9d1bb99dbe23c59`. Commit de captura: `e7dc5a636d0c30f1a47470c2422f8b5524a18ffa`.

**Observação:** 45 interações HTTP; métodos GET: 39, POST: 6.

## Operações HTTP registradas

- `acompanhamento_gerenciar`: 13 requisição(ões)
- `procedimento_visualizar`: 12 requisição(ões)
- `acompanhamento_cadastrar`: 9 requisição(ões)
- `procedimento_gerar`: 3 requisição(ões)
- `procedimento_escolher_tipo`: 2 requisição(ões)
- `acompanhamento_excluir`: 2 requisição(ões)
- `procedimento_controlar`: 1 requisição(ões)
- `procedimento_trabalhar`: 1 requisição(ões)
- `arvore_visualizar`: 1 requisição(ões)
- `procedimento_concluir`: 1 requisição(ões)

## Alcance da evidência

A gravação comprova respostas observadas na instalação indicada. Não demonstra,
por si só, paridade com a CLI IES, corpo de requisições POST ou sucesso sem
conferência pós-escrita. Use os passos relevantes e o contrato antes de implementar.

Consulte [como o SEI funciona](../../docs/como-o-sei-funciona/README.md) e
[o procedimento de replay](../../docs/cassettes.md).
