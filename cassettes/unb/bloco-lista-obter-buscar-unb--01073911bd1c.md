---
type: Cassette
title: bloco-lista-obter-buscar-unb
installation: unb
source_zip: https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-unb.zip
archive_member: bloco-lista-obter-buscar-unb.json
archive_sha1: 21e0559015c56c403daf8933e5040504a7b83837
cassette_sha256_lf: 01073911bd1c2c88e8878c9cc422b0ea536069168feee93f7e4d1a14ecaa0609
interactions: 25
schema_version: 1
complete: true
---

# bloco-lista-obter-buscar-unb

**Instalação:** unb (`sei.treinamento.unb.br`). **Origem:** [ZIP imutável do Internet Archive](https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-unb.zip); membro `bloco-lista-obter-buscar-unb.json`.

**Verificações:** SHA-1 do ZIP `21e0559015c56c403daf8933e5040504a7b83837`; SHA-256 da cassette após CRLF→LF `01073911bd1c2c88e8878c9cc422b0ea536069168feee93f7e4d1a14ecaa0609`. Commit de captura: `63185c4accef370cc477c4d9ba375d3a640235a0`.

**Observação:** 25 interações HTTP; métodos GET: 9, POST: 16.

## Operações HTTP registradas

- `bloco_assinatura_listar`: 23 requisição(ões)
- `procedimento_controlar`: 1 requisição(ões)
- `rel_bloco_protocolo_listar`: 1 requisição(ões)

## Alcance da evidência

A gravação comprova respostas observadas na instalação indicada. Não demonstra,
por si só, paridade com a CLI IES, corpo de requisições POST ou sucesso sem
conferência pós-escrita. Use os passos relevantes e o contrato antes de implementar.

Consulte [como o SEI funciona](../../docs/como-o-sei-funciona/README.md) e
[o procedimento de replay](../../docs/cassettes.md).
