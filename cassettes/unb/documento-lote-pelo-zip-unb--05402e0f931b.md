---
type: Cassette
title: documento-lote-pelo-zip-unb
installation: unb
source_zip: https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-unb.zip
archive_member: documento-lote-pelo-zip-unb.json
archive_sha1: 21e0559015c56c403daf8933e5040504a7b83837
cassette_sha256_lf: 05402e0f931bace345537dc62af12015c3976cb400627b82eb01aa97653596f9
interactions: 13
schema_version: 1
complete: true
---

# documento-lote-pelo-zip-unb

**Instalação:** unb (`sei.treinamento.unb.br`). **Origem:** [ZIP imutável do Internet Archive](https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-unb.zip); membro `documento-lote-pelo-zip-unb.json`.

**Verificações:** SHA-1 do ZIP `21e0559015c56c403daf8933e5040504a7b83837`; SHA-256 da cassette após CRLF→LF `05402e0f931bace345537dc62af12015c3976cb400627b82eb01aa97653596f9`. Commit de captura: `18043caec26fbcf6f20b8d13108347f2116dfa57`.

**Observação:** 13 interações HTTP; métodos GET: 10, POST: 3.

## Operações HTTP registradas

- `procedimento_visualizar`: 2 requisição(ões)
- `procedimento_gerar_zip`: 2 requisição(ões)
- `painel_controle_visualizar`: 1 requisição(ões)
- `procedimento_controlar`: 1 requisição(ões)
- `protocolo_pesquisa_rapida`: 1 requisição(ões)
- `procedimento_trabalhar`: 1 requisição(ões)
- `exibir_arquivo`: 1 requisição(ões)

## Alcance da evidência

A gravação comprova respostas observadas na instalação indicada. Não demonstra,
por si só, paridade com a CLI IES, corpo de requisições POST ou sucesso sem
conferência pós-escrita. Use os passos relevantes e o contrato antes de implementar.

Consulte [como o SEI funciona](../../docs/como-o-sei-funciona/README.md) e
[o procedimento de replay](../../docs/cassettes.md).
