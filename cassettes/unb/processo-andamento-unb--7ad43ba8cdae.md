---
type: Cassette
title: processo-andamento-unb
installation: unb
source_zip: https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-unb.zip
archive_member: processo-andamento-unb.json
archive_sha1: 21e0559015c56c403daf8933e5040504a7b83837
cassette_sha256_lf: 7ad43ba8cdae6054b8e7ae91d19bf13658c563028ed0dfcd0d236e4a7a9ac522
interactions: 26
schema_version: 1
complete: true
---

# processo-andamento-unb

**Instalação:** unb (`sei.treinamento.unb.br`). **Origem:** [ZIP imutável do Internet Archive](https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-unb.zip); membro `processo-andamento-unb.json`.

**Verificações:** SHA-1 do ZIP `21e0559015c56c403daf8933e5040504a7b83837`; SHA-256 da cassette após CRLF→LF `7ad43ba8cdae6054b8e7ae91d19bf13658c563028ed0dfcd0d236e4a7a9ac522`. Commit de captura: `c74e7c5847ac5c08d9d7ce393ef4ea4b3bb59191`.

**Observação:** 26 interações HTTP; métodos GET: 21, POST: 5.

## Operações HTTP registradas

- `procedimento_controlar`: 7 requisição(ões)
- `protocolo_pesquisa_rapida`: 5 requisição(ões)
- `procedimento_trabalhar`: 5 requisição(ões)
- `procedimento_visualizar`: 5 requisição(ões)
- `procedimento_consultar_historico`: 2 requisição(ões)
- `andamento_marcador_gerenciar`: 1 requisição(ões)
- `andamento_marcador_cadastrar`: 1 requisição(ões)

## Alcance da evidência

A gravação comprova respostas observadas na instalação indicada. Não demonstra,
por si só, paridade com a CLI IES, corpo de requisições POST ou sucesso sem
conferência pós-escrita. Use os passos relevantes e o contrato antes de implementar.

Consulte [como o SEI funciona](../../docs/como-o-sei-funciona/README.md) e
[o procedimento de replay](../../docs/cassettes.md).
