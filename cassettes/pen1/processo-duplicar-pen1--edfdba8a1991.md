---
type: Cassette
title: processo-duplicar-pen1
installation: pen1
source_zip: https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-pen1.zip
archive_member: processo-duplicar-pen1.json
archive_sha1: 70841f418106b058c4f81c9b13a1322c51ce8cd0
cassette_sha256_lf: edfdba8a1991655b992b00ff49a9de8e4f0d531f0e40a13475b2a1ba01f0a7e2
interactions: 38
schema_version: 1
complete: true
---

# processo-duplicar-pen1

**Instalação:** pen1 (`sei.orgao1.tramita.processoeletronico.gov.br`). **Origem:** [ZIP imutável do Internet Archive](https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-pen1.zip); membro `processo-duplicar-pen1.json`.

**Verificações:** SHA-1 do ZIP `70841f418106b058c4f81c9b13a1322c51ce8cd0`; SHA-256 da cassette após CRLF→LF `edfdba8a1991655b992b00ff49a9de8e4f0d531f0e40a13475b2a1ba01f0a7e2`. Commit de captura: `4760ede3bb5db5b897ddd40367bdeee4b2d898d7`.

**Observação:** 38 interações HTTP; métodos GET: 32, POST: 6.

## Operações HTTP registradas

- `procedimento_visualizar`: 8 requisição(ões)
- `documento_gerar`: 4 requisição(ões)
- `arvore_visualizar`: 4 requisição(ões)
- `procedimento_duplicar`: 4 requisição(ões)
- `procedimento_gerar`: 2 requisição(ões)
- `procedimento_trabalhar`: 2 requisição(ões)
- `documento_escolher_tipo`: 2 requisição(ões)
- `editor_montar`: 2 requisição(ões)
- `editor_salvar`: 2 requisição(ões)
- `documento_imprimir_web`: 2 requisição(ões)
- `documento_visualizar`: 2 requisição(ões)
- `procedimento_concluir`: 2 requisição(ões)
- `procedimento_controlar`: 1 requisição(ões)
- `procedimento_escolher_tipo`: 1 requisição(ões)

## Alcance da evidência

A gravação comprova respostas observadas na instalação indicada. Não demonstra,
por si só, paridade com a CLI IES, corpo de requisições POST ou sucesso sem
conferência pós-escrita. Use os passos relevantes e o contrato antes de implementar.

Consulte [como o SEI funciona](../../docs/como-o-sei-funciona/README.md) e
[o procedimento de replay](../../docs/cassettes.md).
