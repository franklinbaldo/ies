---
type: Cassette
title: unidade-listar-retornos-unb
installation: unb
source_zip: https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-unb.zip
archive_member: unidade-listar-retornos-unb.json
archive_sha1: 21e0559015c56c403daf8933e5040504a7b83837
cassette_sha256_lf: 61ae9a77b5dff819e31ccf2b92fab7dbdb139b36c1d344533eb38b9e5afec320
interactions: 30
schema_version: 1
complete: true
---

# unidade-listar-retornos-unb

**Instalação:** unb (`sei.treinamento.unb.br`). **Origem:** [ZIP imutável do Internet Archive](https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-unb.zip); membro `unidade-listar-retornos-unb.json`.

**Verificações:** SHA-1 do ZIP `21e0559015c56c403daf8933e5040504a7b83837`; SHA-256 da cassette após CRLF→LF `61ae9a77b5dff819e31ccf2b92fab7dbdb139b36c1d344533eb38b9e5afec320`. Commit de captura: `17e75f7b4108650c196667701e46627530c9ede9`.

**Observação:** 30 interações HTTP; métodos GET: 26, POST: 4.

## Operações HTTP registradas

- `retorno_programado_listar`: 9 requisição(ões)
- `procedimento_visualizar`: 4 requisição(ões)
- `procedimento_gerar`: 3 requisição(ões)
- `procedimento_escolher_tipo`: 2 requisição(ões)
- `procedimento_enviar`: 2 requisição(ões)
- `procedimento_controlar`: 1 requisição(ões)
- `procedimento_trabalhar`: 1 requisição(ões)
- `unidade_auto_completar_envio_processo`: 1 requisição(ões)
- `procedimento_consultar_historico`: 1 requisição(ões)
- `arvore_visualizar`: 1 requisição(ões)
- `procedimento_concluir`: 1 requisição(ões)

## Alcance da evidência

A gravação comprova respostas observadas na instalação indicada. Não demonstra,
por si só, paridade com a CLI IES, corpo de requisições POST ou sucesso sem
conferência pós-escrita. Use os passos relevantes e o contrato antes de implementar.

Consulte [como o SEI funciona](../../docs/como-o-sei-funciona/README.md) e
[o procedimento de replay](../../docs/cassettes.md).
