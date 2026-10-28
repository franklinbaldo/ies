---
type: Cassette
title: processo-acompanhar-unb
installation: unb
source_zip: https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-unb.zip
archive_member: processo-acompanhar-unb.json
archive_sha1: 21e0559015c56c403daf8933e5040504a7b83837
cassette_sha256_lf: 4e19a6146bc49c73103c57ea475eb12dfc86edad12d483c981f3244da6e036bd
interactions: 45
schema_version: 1
complete: true
---

# processo-acompanhar-unb

**Instalação:** unb (`sei.treinamento.unb.br`). **Origem:** [ZIP imutável do Internet Archive](https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-unb.zip); membro `processo-acompanhar-unb.json`.

**Verificações:** SHA-1 do ZIP `21e0559015c56c403daf8933e5040504a7b83837`; SHA-256 da cassette após CRLF→LF `4e19a6146bc49c73103c57ea475eb12dfc86edad12d483c981f3244da6e036bd`. Commit de captura: `c514ec5e4405a37664507f9038dbd7649ac497d0`.

**Observação:** 45 interações HTTP; métodos GET: 39, POST: 6.

## Operações HTTP registradas

- `acompanhamento_gerenciar`: 13 requisição(ões)
- `procedimento_visualizar`: 12 requisição(ões)
- `acompanhamento_cadastrar`: 9 requisição(ões)
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
