---
type: Cassette
title: unidade-listar-retornos-pen4
installation: pen4
source_zip: https://archive.org/download/todos-sei-cassettes-2026/lotes/lote-1e2f128fe845906a.zip
archive_member: unidade-listar-retornos-pen4.json
archive_sha1: 1e2f128fe845906a364058f0d5eb22a0308b4095
cassette_sha256_lf: 87e2e8e70f44563f4e02345986c7b598387feecc82a3b8bf452290ddb128d14a
interactions: 28
schema_version: 1
complete: true
---

# unidade-listar-retornos-pen4

**Instalação:** pen4 (`sei.orgao4.tramita.processoeletronico.gov.br`). **Origem:** [ZIP imutável do Internet Archive](https://archive.org/download/todos-sei-cassettes-2026/lotes/lote-1e2f128fe845906a.zip); membro `unidade-listar-retornos-pen4.json`.

**Verificações:** SHA-1 do ZIP `1e2f128fe845906a364058f0d5eb22a0308b4095`; SHA-256 da cassette após CRLF→LF `87e2e8e70f44563f4e02345986c7b598387feecc82a3b8bf452290ddb128d14a`. Commit de captura: `a87c068bb2fc80cf62d9ff689e44adf7a70faeef`.

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
