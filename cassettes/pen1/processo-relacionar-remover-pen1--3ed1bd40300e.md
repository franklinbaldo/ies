---
type: Cassette
title: processo-relacionar-remover-pen1
installation: pen1
source_zip: https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-pen1.zip
archive_member: processo-relacionar-remover-pen1.json
archive_sha1: 70841f418106b058c4f81c9b13a1322c51ce8cd0
cassette_sha256_lf: 3ed1bd40300e4294d72b1ff974eb7ba0251cdc41c700df7829725e2a86281ac5
interactions: 25
schema_version: 1
complete: true
---

# processo-relacionar-remover-pen1

**Instalação:** pen1 (`sei.orgao1.tramita.processoeletronico.gov.br`). **Origem:** [ZIP imutável do Internet Archive](https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-pen1.zip); membro `processo-relacionar-remover-pen1.json`.

**Verificações:** SHA-1 do ZIP `70841f418106b058c4f81c9b13a1322c51ce8cd0`; SHA-256 da cassette após CRLF→LF `3ed1bd40300e4294d72b1ff974eb7ba0251cdc41c700df7829725e2a86281ac5`. Commit de captura: `e41486a6ae94625964619a7e309846db5fa3a37d`.

**Observação:** 25 interações HTTP; métodos GET: 22, POST: 3.

## Operações HTTP registradas

- `procedimento_visualizar`: 6 requisição(ões)
- `procedimento_relacionar`: 5 requisição(ões)
- `procedimento_gerar`: 2 requisição(ões)
- `procedimento_trabalhar`: 2 requisição(ões)
- `procedimento_gerar_relacionado`: 2 requisição(ões)
- `arvore_visualizar`: 2 requisição(ões)
- `procedimento_concluir`: 2 requisição(ões)
- `procedimento_controlar`: 1 requisição(ões)
- `procedimento_escolher_tipo`: 1 requisição(ões)
- `procedimento_escolher_tipo_relacionado`: 1 requisição(ões)
- `procedimento_excluir_relacionamento`: 1 requisição(ões)

## Alcance da evidência

A gravação comprova respostas observadas na instalação indicada. Não demonstra,
por si só, paridade com a CLI IES, corpo de requisições POST ou sucesso sem
conferência pós-escrita. Use os passos relevantes e o contrato antes de implementar.

Consulte [como o SEI funciona](../../docs/como-o-sei-funciona/README.md) e
[o procedimento de replay](../../docs/cassettes.md).
