---
type: Cassette
title: processo-excluir-pen1
installation: pen1
source_zip: https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-pen1.zip
archive_member: processo-excluir-pen1.json
archive_sha1: 70841f418106b058c4f81c9b13a1322c51ce8cd0
cassette_sha256_lf: fc4c8d59a2b2ec491c96ef3f5d9cf182d504d02b5f48c36932818dee11e1897f
interactions: 36
schema_version: 1
complete: true
---

# processo-excluir-pen1

**Instalação:** pen1 (`sei.orgao1.tramita.processoeletronico.gov.br`). **Origem:** [ZIP imutável do Internet Archive](https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-pen1.zip); membro `processo-excluir-pen1.json`.

**Verificações:** SHA-1 do ZIP `70841f418106b058c4f81c9b13a1322c51ce8cd0`; SHA-256 da cassette após CRLF→LF `fc4c8d59a2b2ec491c96ef3f5d9cf182d504d02b5f48c36932818dee11e1897f`. Commit de captura: `94c58f8609d128bfbb92c5c79d999c1c50b82f47`.

**Observação:** 36 interações HTTP; métodos GET: 30, POST: 6.

## Operações HTTP registradas

- `procedimento_visualizar`: 9 requisição(ões)
- `procedimento_trabalhar`: 6 requisição(ões)
- `procedimento_gerar`: 4 requisição(ões)
- `arvore_visualizar`: 3 requisição(ões)
- `procedimento_escolher_tipo`: 2 requisição(ões)
- `protocolo_pesquisa_rapida`: 2 requisição(ões)
- `documento_gerar`: 2 requisição(ões)
- `procedimento_controlar`: 1 requisição(ões)
- `procedimento_excluir`: 1 requisição(ões)
- `documento_escolher_tipo`: 1 requisição(ões)
- `editor_montar`: 1 requisição(ões)
- `editor_salvar`: 1 requisição(ões)
- `documento_imprimir_web`: 1 requisição(ões)
- `documento_visualizar`: 1 requisição(ões)
- `procedimento_concluir`: 1 requisição(ões)

## Alcance da evidência

A gravação comprova respostas observadas na instalação indicada. Não demonstra,
por si só, paridade com a CLI IES, corpo de requisições POST ou sucesso sem
conferência pós-escrita. Use os passos relevantes e o contrato antes de implementar.

Consulte [como o SEI funciona](../../docs/como-o-sei-funciona/README.md) e
[o procedimento de replay](../../docs/cassettes.md).
