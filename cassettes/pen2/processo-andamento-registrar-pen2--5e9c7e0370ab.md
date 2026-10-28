---
type: Cassette
title: processo-andamento-registrar-pen2
installation: pen2
source_zip: https://archive.org/download/todos-sei-cassettes-2026/lotes/lote-0306b45276ae0084.zip
archive_member: processo-andamento-registrar-pen2.json
archive_sha1: 0306b45276ae0084b78e781051f7ab5fc07e2693
cassette_sha256_lf: 5e9c7e0370abd45ef69d78dbee6a4b65b6513445c28444b3bdff1f791827712d
interactions: 25
schema_version: 1
complete: true
---

# processo-andamento-registrar-pen2

**Instalação:** pen2 (`sei.orgao2.tramita.processoeletronico.gov.br`). **Origem:** [ZIP imutável do Internet Archive](https://archive.org/download/todos-sei-cassettes-2026/lotes/lote-0306b45276ae0084.zip); membro `processo-andamento-registrar-pen2.json`.

**Verificações:** SHA-1 do ZIP `0306b45276ae0084b78e781051f7ab5fc07e2693`; SHA-256 da cassette após CRLF→LF `5e9c7e0370abd45ef69d78dbee6a4b65b6513445c28444b3bdff1f791827712d`. Commit de captura: `c5656af8d6b9447c10a35e40910a41ad25cdc99a`.

**Observação:** 25 interações HTTP; métodos GET: 19, POST: 6.

## Operações HTTP registradas

- `procedimento_visualizar`: 6 requisição(ões)
- `procedimento_atualizar_andamento`: 5 requisição(ões)
- `procedimento_escolher_tipo`: 4 requisição(ões)
- `procedimento_gerar`: 3 requisição(ões)
- `procedimento_consultar_historico`: 3 requisição(ões)
- `procedimento_concluir`: 2 requisição(ões)
- `procedimento_controlar`: 1 requisição(ões)
- `procedimento_trabalhar`: 1 requisição(ões)

## Alcance da evidência

A gravação comprova respostas observadas na instalação indicada. Não demonstra,
por si só, paridade com a CLI IES, corpo de requisições POST ou sucesso sem
conferência pós-escrita. Use os passos relevantes e o contrato antes de implementar.

Consulte [como o SEI funciona](../../docs/como-o-sei-funciona/README.md) e
[o procedimento de replay](../../docs/cassettes.md).
