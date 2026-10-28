---
type: Cassette
title: bloco-alterar-unb
installation: unb
source_zip: https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-unb.zip
archive_member: bloco-alterar-unb.json
archive_sha1: 21e0559015c56c403daf8933e5040504a7b83837
cassette_sha256_lf: d36a8c8725eb02aae58f6b1481af72a7b0e6e7d7ef578cce184fc3653b825a25
interactions: 31
schema_version: 1
complete: true
---

# bloco-alterar-unb

**Instalação:** unb (`sei.treinamento.unb.br`). **Origem:** [ZIP imutável do Internet Archive](https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-unb.zip); membro `bloco-alterar-unb.json`.

**Verificações:** SHA-1 do ZIP `21e0559015c56c403daf8933e5040504a7b83837`; SHA-256 da cassette após CRLF→LF `d36a8c8725eb02aae58f6b1481af72a7b0e6e7d7ef578cce184fc3653b825a25`. Commit de captura: `a917dcb53a044c9b3d5188ca2fec98f0a79c27d9`.

**Observação:** 31 interações HTTP; métodos GET: 23, POST: 8.

## Operações HTTP registradas

- `bloco_assinatura_listar`: 16 requisição(ões)
- `bloco_assinatura_alterar`: 8 requisição(ões)
- `bloco_assinatura_cadastrar`: 3 requisição(ões)
- `unidade_auto_completar_outras`: 2 requisição(ões)
- `procedimento_controlar`: 1 requisição(ões)
- `bloco_excluir`: 1 requisição(ões)

## Alcance da evidência

A gravação comprova respostas observadas na instalação indicada. Não demonstra,
por si só, paridade com a CLI IES, corpo de requisições POST ou sucesso sem
conferência pós-escrita. Use os passos relevantes e o contrato antes de implementar.

Consulte [como o SEI funciona](../../docs/como-o-sei-funciona/README.md) e
[o procedimento de replay](../../docs/cassettes.md).
