---
type: Cassette
title: unidade-listar-retornos-pen2
installation: pen2
source_zip: https://archive.org/download/todos-sei-cassettes-2026/lotes/lote-28a245e07bf7d358.zip
archive_member: unidade-listar-retornos-pen2.json
archive_sha1: 28a245e07bf7d358a322981744dc5e79a835ba94
cassette_sha256_lf: 8bbd8541acd6e1dbe38f194978069c8ef60ac1201c7adbe06fc4b1e221368026
interactions: 28
schema_version: 1
complete: true
---

# unidade-listar-retornos-pen2

**Instalação:** pen2 (`sei.orgao2.tramita.processoeletronico.gov.br`). **Origem:** [ZIP imutável do Internet Archive](https://archive.org/download/todos-sei-cassettes-2026/lotes/lote-28a245e07bf7d358.zip); membro `unidade-listar-retornos-pen2.json`.

**Verificações:** SHA-1 do ZIP `28a245e07bf7d358a322981744dc5e79a835ba94`; SHA-256 da cassette após CRLF→LF `8bbd8541acd6e1dbe38f194978069c8ef60ac1201c7adbe06fc4b1e221368026`. Commit de captura: `a1345f79e3e1202cc39a0c82b2d063c73b72c131`.

**Observação:** 28 interações HTTP; métodos GET: 22, POST: 6.

## Operações HTTP registradas

- `retorno_programado_listar`: 9 requisição(ões)
- `procedimento_escolher_tipo`: 4 requisição(ões)
- `procedimento_visualizar`: 4 requisição(ões)
- `procedimento_gerar`: 3 requisição(ões)
- `procedimento_enviar`: 2 requisição(ões)
- `procedimento_concluir`: 2 requisição(ões)
- `procedimento_controlar`: 1 requisição(ões)
- `procedimento_trabalhar`: 1 requisição(ões)
- `unidade_auto_completar_envio_processo`: 1 requisição(ões)
- `procedimento_consultar_historico`: 1 requisição(ões)

## Alcance da evidência

A gravação comprova respostas observadas na instalação indicada. Não demonstra,
por si só, paridade com a CLI IES, corpo de requisições POST ou sucesso sem
conferência pós-escrita. Use os passos relevantes e o contrato antes de implementar.

Consulte [como o SEI funciona](../../docs/como-o-sei-funciona/README.md) e
[o procedimento de replay](../../docs/cassettes.md).
