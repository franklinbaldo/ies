---
type: Cassette
title: processo-andamento-pen1
installation: pen1
source_zip: https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-pen1.zip
archive_member: processo-andamento-pen1.json
archive_sha1: 70841f418106b058c4f81c9b13a1322c51ce8cd0
cassette_sha256_lf: 883fd03d8c51f788516195915230cafa9ba4cf6af9df4b45b99bc11f4c8ceea2
interactions: 11
schema_version: 1
complete: true
---

# processo-andamento-pen1

**Instalação:** pen1 (`sei.orgao1.tramita.processoeletronico.gov.br`). **Origem:** [ZIP imutável do Internet Archive](https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-pen1.zip); membro `processo-andamento-pen1.json`.

**Verificações:** SHA-1 do ZIP `70841f418106b058c4f81c9b13a1322c51ce8cd0`; SHA-256 da cassette após CRLF→LF `883fd03d8c51f788516195915230cafa9ba4cf6af9df4b45b99bc11f4c8ceea2`. Commit de captura: `86bfe7c5b777c62f4c283ea7d17fd2ec9e9a761e`.

**Observação:** 11 interações HTTP; métodos GET: 10, POST: 1.

## Operações HTTP registradas

- `procedimento_controlar`: 1 requisição(ões)
- `protocolo_pesquisa_rapida`: 1 requisição(ões)
- `procedimento_trabalhar`: 1 requisição(ões)
- `procedimento_visualizar`: 1 requisição(ões)
- `andamento_marcador_gerenciar`: 1 requisição(ões)
- `andamento_marcador_cadastrar`: 1 requisição(ões)
- `procedimento_alterar`: 1 requisição(ões)
- `anotacao_registrar`: 1 requisição(ões)
- `procedimento_atribuicao_cadastrar`: 1 requisição(ões)
- `procedimento_relacionar`: 1 requisição(ões)
- `procedimento_consultar_historico`: 1 requisição(ões)

## Alcance da evidência

A gravação comprova respostas observadas na instalação indicada. Não demonstra,
por si só, paridade com a CLI IES, corpo de requisições POST ou sucesso sem
conferência pós-escrita. Use os passos relevantes e o contrato antes de implementar.

Consulte [como o SEI funciona](../../docs/como-o-sei-funciona/README.md) e
[o procedimento de replay](../../docs/cassettes.md).
