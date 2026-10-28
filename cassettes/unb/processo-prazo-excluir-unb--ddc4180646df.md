---
type: Cassette
title: processo-prazo-excluir-unb
installation: unb
source_zip: https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-unb.zip
archive_member: processo-prazo-excluir-unb.json
archive_sha1: 21e0559015c56c403daf8933e5040504a7b83837
cassette_sha256_lf: ddc4180646dfafcfbbe6a04bcf3a3240b41dc0e5dc99c7d6b71dd02736f9d41c
interactions: 23
schema_version: 1
complete: true
---

# processo-prazo-excluir-unb

**Instalação:** unb (`sei.treinamento.unb.br`). **Origem:** [ZIP imutável do Internet Archive](https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-unb.zip); membro `processo-prazo-excluir-unb.json`.

**Verificações:** SHA-1 do ZIP `21e0559015c56c403daf8933e5040504a7b83837`; SHA-256 da cassette após CRLF→LF `ddc4180646dfafcfbbe6a04bcf3a3240b41dc0e5dc99c7d6b71dd02736f9d41c`. Commit de captura: `55352608cce56433e654d1108f20363f86a654b7`.

**Observação:** 23 interações HTTP; métodos GET: 20, POST: 3.

## Operações HTTP registradas

- `procedimento_visualizar`: 5 requisição(ões)
- `controle_prazo_listar`: 5 requisição(ões)
- `procedimento_gerar`: 3 requisição(ões)
- `controle_prazo_definir`: 3 requisição(ões)
- `procedimento_escolher_tipo`: 2 requisição(ões)
- `procedimento_controlar`: 1 requisição(ões)
- `procedimento_trabalhar`: 1 requisição(ões)
- `controle_prazo_excluir`: 1 requisição(ões)
- `arvore_visualizar`: 1 requisição(ões)
- `procedimento_concluir`: 1 requisição(ões)

## Alcance da evidência

A gravação comprova respostas observadas na instalação indicada. Não demonstra,
por si só, paridade com a CLI IES, corpo de requisições POST ou sucesso sem
conferência pós-escrita. Use os passos relevantes e o contrato antes de implementar.

Consulte [como o SEI funciona](../../docs/como-o-sei-funciona/README.md) e
[o procedimento de replay](../../docs/cassettes.md).
