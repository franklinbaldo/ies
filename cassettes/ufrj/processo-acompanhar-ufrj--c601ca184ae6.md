---
type: Cassette
title: processo-acompanhar-ufrj
installation: ufrj
source_zip: https://archive.org/download/todos-sei-cassettes-2026/lotes/lote-c7a4af8d9d70ddc3.zip
archive_member: processo-acompanhar-ufrj.json
archive_sha1: c7a4af8d9d70ddc3599cdd9bccbb98e6d726db78
cassette_sha256_lf: c601ca184ae6a5b1ac3d29220de513acd9abc4af3f6b813e4c32906b3666bdc4
interactions: 47
schema_version: 1
complete: true
---

# processo-acompanhar-ufrj

**Instalação:** ufrj (`teste.sei.tic.ufrj.br`). **Origem:** [ZIP imutável do Internet Archive](https://archive.org/download/todos-sei-cassettes-2026/lotes/lote-c7a4af8d9d70ddc3.zip); membro `processo-acompanhar-ufrj.json`.

**Verificações:** SHA-1 do ZIP `c7a4af8d9d70ddc3599cdd9bccbb98e6d726db78`; SHA-256 da cassette após CRLF→LF `c601ca184ae6a5b1ac3d29220de513acd9abc4af3f6b813e4c32906b3666bdc4`. Commit de captura: `4118350169982f7b22563c7343dbf600844d65e1`.

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
