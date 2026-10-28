---
type: Cassette
title: processo-acompanhar-alterar-unb
installation: unb
source_zip: https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-unb.zip
archive_member: processo-acompanhar-alterar-unb.json
archive_sha1: 21e0559015c56c403daf8933e5040504a7b83837
cassette_sha256_lf: ae067b73d1df7aca7456c04aafde66406b52b2aeddf9ae62d20da02ce3df2468
interactions: 52
schema_version: 1
complete: true
---

# processo-acompanhar-alterar-unb

**Instalação:** unb (`sei.treinamento.unb.br`). **Origem:** [ZIP imutável do Internet Archive](https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-unb.zip); membro `processo-acompanhar-alterar-unb.json`.

**Verificações:** SHA-1 do ZIP `21e0559015c56c403daf8933e5040504a7b83837`; SHA-256 da cassette após CRLF→LF `ae067b73d1df7aca7456c04aafde66406b52b2aeddf9ae62d20da02ce3df2468`. Commit de captura: `8d3e96ff571b09c3d04ab20ea290b9bb445b8c5d`.

**Observação:** 52 interações HTTP; métodos GET: 43, POST: 9.

## Operações HTTP registradas

- `acompanhamento_gerenciar`: 14 requisição(ões)
- `procedimento_visualizar`: 11 requisição(ões)
- `acompanhamento_alterar`: 7 requisição(ões)
- `acompanhamento_cadastrar`: 5 requisição(ões)
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
