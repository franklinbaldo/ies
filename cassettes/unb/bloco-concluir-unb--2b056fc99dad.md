---
type: Cassette
title: bloco-concluir-unb
installation: unb
source_zip: https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-unb.zip
archive_member: bloco-concluir-unb.json
archive_sha1: 21e0559015c56c403daf8933e5040504a7b83837
cassette_sha256_lf: 2b056fc99dad06989b007739e804369a70b7039e44d25561b0e06dcc1c3ffaf1
interactions: 32
schema_version: 1
complete: true
---

# bloco-concluir-unb

**Instalação:** unb (`sei.treinamento.unb.br`). **Origem:** [ZIP imutável do Internet Archive](https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-unb.zip); membro `bloco-concluir-unb.json`.

**Verificações:** SHA-1 do ZIP `21e0559015c56c403daf8933e5040504a7b83837`; SHA-256 da cassette após CRLF→LF `2b056fc99dad06989b007739e804369a70b7039e44d25561b0e06dcc1c3ffaf1`. Commit de captura: `5f1f4f37bbdf993b5275dc312c4a9bbf72a4e60d`.

**Observação:** 32 interações HTTP; métodos GET: 19, POST: 13.

## Operações HTTP registradas

- `bloco_assinatura_listar`: 25 requisição(ões)
- `bloco_assinatura_cadastrar`: 3 requisição(ões)
- `procedimento_controlar`: 1 requisição(ões)
- `bloco_concluir`: 1 requisição(ões)
- `bloco_reabrir`: 1 requisição(ões)
- `bloco_excluir`: 1 requisição(ões)

## Alcance da evidência

A gravação comprova respostas observadas na instalação indicada. Não demonstra,
por si só, paridade com a CLI IES, corpo de requisições POST ou sucesso sem
conferência pós-escrita. Use os passos relevantes e o contrato antes de implementar.

Consulte [como o SEI funciona](../../docs/como-o-sei-funciona/README.md) e
[o procedimento de replay](../../docs/cassettes.md).
