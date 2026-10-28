---
type: Cassette
title: processo-enviar-email-pen1
installation: pen1
source_zip: https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-pen1.zip
archive_member: processo-enviar-email-pen1.json
archive_sha1: 70841f418106b058c4f81c9b13a1322c51ce8cd0
cassette_sha256_lf: e496d96004b2ae7c8a774cd01538a5a489ea79feffd347bac83bc8c5c880b460
interactions: 21
schema_version: 1
complete: true
---

# processo-enviar-email-pen1

**Instalação:** pen1 (`sei.orgao1.tramita.processoeletronico.gov.br`). **Origem:** [ZIP imutável do Internet Archive](https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-pen1.zip); membro `processo-enviar-email-pen1.json`.

**Verificações:** SHA-1 do ZIP `70841f418106b058c4f81c9b13a1322c51ce8cd0`; SHA-256 da cassette após CRLF→LF `e496d96004b2ae7c8a774cd01538a5a489ea79feffd347bac83bc8c5c880b460`. Commit de captura: `ee9aecea6f226a02b065d6c4d189bd1bde99757d`.

**Observação:** 21 interações HTTP; métodos GET: 19, POST: 2.

## Operações HTTP registradas

- `procedimento_visualizar`: 7 requisição(ões)
- `arvore_visualizar`: 3 requisição(ões)
- `procedimento_enviar_email`: 3 requisição(ões)
- `procedimento_gerar`: 2 requisição(ões)
- `procedimento_consultar_historico`: 2 requisição(ões)
- `procedimento_controlar`: 1 requisição(ões)
- `procedimento_escolher_tipo`: 1 requisição(ões)
- `procedimento_trabalhar`: 1 requisição(ões)
- `procedimento_concluir`: 1 requisição(ões)

## Alcance da evidência

A gravação comprova respostas observadas na instalação indicada. Não demonstra,
por si só, paridade com a CLI IES, corpo de requisições POST ou sucesso sem
conferência pós-escrita. Use os passos relevantes e o contrato antes de implementar.

Consulte [como o SEI funciona](../../docs/como-o-sei-funciona/README.md) e
[o procedimento de replay](../../docs/cassettes.md).
