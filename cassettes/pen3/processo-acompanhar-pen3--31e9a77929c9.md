---
type: Cassette
title: processo-acompanhar-pen3
installation: pen3
source_zip: https://archive.org/download/todos-sei-cassettes-2026/lotes/lote-15a748cdb7c0fdbd.zip
archive_member: processo-acompanhar-pen3.json
archive_sha1: 15a748cdb7c0fdbd907c8b3b922fa0194ae8887c
cassette_sha256_lf: 31e9a77929c96c80dec3ac2337be0af35182590fb304ad9b3ecc785e8b216e87
interactions: 47
schema_version: 1
complete: true
---

# processo-acompanhar-pen3

**Instalação:** pen3 (`sei.orgao3.tramita.processoeletronico.gov.br`). **Origem:** [ZIP imutável do Internet Archive](https://archive.org/download/todos-sei-cassettes-2026/lotes/lote-15a748cdb7c0fdbd.zip); membro `processo-acompanhar-pen3.json`.

**Verificações:** SHA-1 do ZIP `15a748cdb7c0fdbd907c8b3b922fa0194ae8887c`; SHA-256 da cassette após CRLF→LF `31e9a77929c96c80dec3ac2337be0af35182590fb304ad9b3ecc785e8b216e87`. Commit de captura: `9cef611bdb4d89106b3782fc8ea13a2426bd1d4d`.

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
