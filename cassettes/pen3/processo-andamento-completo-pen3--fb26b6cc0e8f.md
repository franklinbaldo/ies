---
type: Cassette
title: processo-andamento-completo-pen3
installation: pen3
source_zip: https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-pen3.zip
archive_member: processo-andamento-completo-pen3.json
archive_sha1: b81b84fca29299f2096fb128593ba52f2ac9a7ab
cassette_sha256_lf: fb26b6cc0e8f649f07bffa68547af3c84f1f20c45361a7f4ef2feb47d3463e13
interactions: 16
schema_version: 1
complete: true
---

# processo-andamento-completo-pen3

**Instalação:** pen3 (`sei.orgao3.tramita.processoeletronico.gov.br`). **Origem:** [ZIP imutável do Internet Archive](https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-pen3.zip); membro `processo-andamento-completo-pen3.json`.

**Verificações:** SHA-1 do ZIP `b81b84fca29299f2096fb128593ba52f2ac9a7ab`; SHA-256 da cassette após CRLF→LF `fb26b6cc0e8f649f07bffa68547af3c84f1f20c45361a7f4ef2feb47d3463e13`. Commit de captura: `1301298fd4ea22efd7431ea231f3da0d27fd621f`.

**Observação:** 16 interações HTTP; métodos GET: 14, POST: 2.

## Operações HTTP registradas

- `procedimento_consultar_historico`: 3 requisição(ões)
- `procedimento_visualizar`: 2 requisição(ões)
- `procedimento_alterar`: 2 requisição(ões)
- `anotacao_registrar`: 2 requisição(ões)
- `procedimento_atribuicao_cadastrar`: 2 requisição(ões)
- `procedimento_relacionar`: 2 requisição(ões)
- `procedimento_controlar`: 1 requisição(ões)
- `protocolo_pesquisa_rapida`: 1 requisição(ões)
- `procedimento_trabalhar`: 1 requisição(ões)

## Alcance da evidência

A gravação comprova respostas observadas na instalação indicada. Não demonstra,
por si só, paridade com a CLI IES, corpo de requisições POST ou sucesso sem
conferência pós-escrita. Use os passos relevantes e o contrato antes de implementar.

Consulte [como o SEI funciona](../../docs/como-o-sei-funciona/README.md) e
[o procedimento de replay](../../docs/cassettes.md).
