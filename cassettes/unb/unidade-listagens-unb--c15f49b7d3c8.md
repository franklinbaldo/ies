---
type: Cassette
title: unidade-listagens-unb
installation: unb
source_zip: https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-unb.zip
archive_member: unidade-listagens-unb.json
archive_sha1: 21e0559015c56c403daf8933e5040504a7b83837
cassette_sha256_lf: c15f49b7d3c8982d96bea9aefbcbf262c1949cd7c81bd495d53100cf29293604
interactions: 15
schema_version: 1
complete: true
---

# unidade-listagens-unb

**Instalação:** unb (`sei.treinamento.unb.br`). **Origem:** [ZIP imutável do Internet Archive](https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-unb.zip); membro `unidade-listagens-unb.json`.

**Verificações:** SHA-1 do ZIP `21e0559015c56c403daf8933e5040504a7b83837`; SHA-256 da cassette após CRLF→LF `c15f49b7d3c8982d96bea9aefbcbf262c1949cd7c81bd495d53100cf29293604`. Commit de captura: `f9328866a91750db5d8e9935d2f28d146bf18b98`.

**Observação:** 15 interações HTTP; métodos GET: 7, POST: 8.

## Operações HTTP registradas

- `bloco_assinatura_listar`: 8 requisição(ões)
- `acompanhamento_listar`: 3 requisição(ões)
- `procedimento_controlar`: 1 requisição(ões)
- `controle_prazo_listar`: 1 requisição(ões)
- `procedimento_sobrestado_listar`: 1 requisição(ões)
- `texto_padrao_interno_listar`: 1 requisição(ões)

## Alcance da evidência

A gravação comprova respostas observadas na instalação indicada. Não demonstra,
por si só, paridade com a CLI IES, corpo de requisições POST ou sucesso sem
conferência pós-escrita. Use os passos relevantes e o contrato antes de implementar.

Consulte [como o SEI funciona](../../docs/como-o-sei-funciona/README.md) e
[o procedimento de replay](../../docs/cassettes.md).
