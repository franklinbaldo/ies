---
type: Cassette
title: unidade-listar-retornos-ufrj
installation: ufrj
source_zip: https://archive.org/download/todos-sei-cassettes-2026/lotes/lote-c7a4af8d9d70ddc3.zip
archive_member: unidade-listar-retornos-ufrj.json
archive_sha1: c7a4af8d9d70ddc3599cdd9bccbb98e6d726db78
cassette_sha256_lf: 3a6152f7d1bc733b4f3310151766816e3934df2d1b23348ddf8b1a344a808110
interactions: 28
schema_version: 1
complete: true
---

# unidade-listar-retornos-ufrj

**Instalação:** ufrj (`teste.sei.tic.ufrj.br`). **Origem:** [ZIP imutável do Internet Archive](https://archive.org/download/todos-sei-cassettes-2026/lotes/lote-c7a4af8d9d70ddc3.zip); membro `unidade-listar-retornos-ufrj.json`.

**Verificações:** SHA-1 do ZIP `c7a4af8d9d70ddc3599cdd9bccbb98e6d726db78`; SHA-256 da cassette após CRLF→LF `3a6152f7d1bc733b4f3310151766816e3934df2d1b23348ddf8b1a344a808110`. Commit de captura: `0c2b0b520d8da2bc8d84cd9a83837083481922e2`.

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
