---
type: Cassette
title: processo-andamento-completo-ufrj
installation: ufrj
source_zip: https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-ufrj.zip
archive_member: processo-andamento-completo-ufrj.json
archive_sha1: 3b29ba3c4e18fa0b336fbec9490c4f59a15d4aa0
cassette_sha256_lf: 6154b2b9d7c964f199e34b67d0be14ef6c3273ac423e273d3b0d662ed24c5b3a
interactions: 16
schema_version: 1
complete: true
---

# processo-andamento-completo-ufrj

**Instalação:** ufrj (`teste.sei.tic.ufrj.br`). **Origem:** [ZIP imutável do Internet Archive](https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-ufrj.zip); membro `processo-andamento-completo-ufrj.json`.

**Verificações:** SHA-1 do ZIP `3b29ba3c4e18fa0b336fbec9490c4f59a15d4aa0`; SHA-256 da cassette após CRLF→LF `6154b2b9d7c964f199e34b67d0be14ef6c3273ac423e273d3b0d662ed24c5b3a`. Commit de captura: `6f8c395048d38e11b86b926215c0f2a254bbd9ee`.

**Observação:** 16 interações HTTP; métodos GET: 14, POST: 2.

## Operações HTTP registradas

- `procedimento_consultar_historico`: 3 requisição(ões)
- `procedimento_visualizar`: 2 requisição(ões)
- `procedimento_alterar`: 2 requisição(ões)
- `anotacao_registrar`: 2 requisição(ões)
- `procedimento_atribuicao_cadastrar`: 2 requisição(ões)
- `procedimento_relacionar`: 2 requisição(ões)
- `procedimento_controlar`: 1 requisição(ões)
- `protocolo_pesquisa_rapida`: 1 requisição(ões)
- `procedimento_trabalhar`: 1 requisição(ões)

## Alcance da evidência

A gravação comprova respostas observadas na instalação indicada. Não demonstra,
por si só, paridade com a CLI IES, corpo de requisições POST ou sucesso sem
conferência pós-escrita. Use os passos relevantes e o contrato antes de implementar.

Consulte [como o SEI funciona](../../docs/como-o-sei-funciona/README.md) e
[o procedimento de replay](../../docs/cassettes.md).
