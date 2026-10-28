---
type: Cassette
title: processo-atribuir-unb
installation: unb
source_zip: https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-unb.zip
archive_member: processo-atribuir-unb.json
archive_sha1: 21e0559015c56c403daf8933e5040504a7b83837
cassette_sha256_lf: bf0d4327d7d9de550b128c2d7fe5e4f282aab65a7e20eab548952fe89b051b20
interactions: 24
schema_version: 1
complete: true
---

# processo-atribuir-unb

**Instalação:** unb (`sei.treinamento.unb.br`). **Origem:** [ZIP imutável do Internet Archive](https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-unb.zip); membro `processo-atribuir-unb.json`.

**Verificações:** SHA-1 do ZIP `21e0559015c56c403daf8933e5040504a7b83837`; SHA-256 da cassette após CRLF→LF `bf0d4327d7d9de550b128c2d7fe5e4f282aab65a7e20eab548952fe89b051b20`. Commit de captura: `4eaf98af079c12176d77611bfbca6036d4104521`.

**Observação:** 24 interações HTTP; métodos GET: 21, POST: 3.

## Operações HTTP registradas

- `procedimento_visualizar`: 9 requisição(ões)
- `procedimento_atribuicao_cadastrar`: 6 requisição(ões)
- `procedimento_gerar`: 3 requisição(ões)
- `procedimento_escolher_tipo`: 2 requisição(ões)
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
