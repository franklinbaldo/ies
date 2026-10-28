---
type: Cassette
title: documento-mover-cancelar-unb
installation: unb
source_zip: https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-unb.zip
archive_member: documento-mover-cancelar-unb.json
archive_sha1: 21e0559015c56c403daf8933e5040504a7b83837
cassette_sha256_lf: 0c37c22872c9ca649c8106d78c34690e14a3ef76e6e7dc757d35d4be9706a556
interactions: 60
schema_version: 1
complete: true
---

# documento-mover-cancelar-unb

**Instalação:** unb (`sei.treinamento.unb.br`). **Origem:** [ZIP imutável do Internet Archive](https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-unb.zip); membro `documento-mover-cancelar-unb.json`.

**Verificações:** SHA-1 do ZIP `21e0559015c56c403daf8933e5040504a7b83837`; SHA-256 da cassette após CRLF→LF `0c37c22872c9ca649c8106d78c34690e14a3ef76e6e7dc757d35d4be9706a556`. Commit de captura: `7593dfdf021d754c733564448ff684324e90847f`.

**Observação:** 60 interações HTTP; métodos GET: 46, POST: 14.

## Operações HTTP registradas

- `procedimento_visualizar`: 15 requisição(ões)
- `procedimento_gerar`: 6 requisição(ões)
- `documento_receber`: 5 requisição(ões)
- `procedimento_escolher_tipo`: 4 requisição(ões)
- `arvore_visualizar`: 4 requisição(ões)
- `documento_escolher_tipo`: 3 requisição(ões)
- `documento_recebido_duplicado`: 3 requisição(ões)
- `documento_imprimir_web`: 3 requisição(ões)
- `procedimento_trabalhar`: 2 requisição(ões)
- `documento_gerar`: 2 requisição(ões)
- `documento_assinar`: 2 requisição(ões)
- `procedimento_enviar`: 2 requisição(ões)
- `procedimento_concluir`: 2 requisição(ões)
- `procedimento_controlar`: 1 requisição(ões)
- `documento_upload_anexo`: 1 requisição(ões)
- `editor_montar`: 1 requisição(ões)
- `editor_salvar`: 1 requisição(ões)
- `documento_visualizar`: 1 requisição(ões)
- `unidade_auto_completar_envio_processo`: 1 requisição(ões)
- `procedimento_consultar_historico`: 1 requisição(ões)

## Alcance da evidência

A gravação comprova respostas observadas na instalação indicada. Não demonstra,
por si só, paridade com a CLI IES, corpo de requisições POST ou sucesso sem
conferência pós-escrita. Use os passos relevantes e o contrato antes de implementar.

Consulte [como o SEI funciona](../../docs/como-o-sei-funciona/README.md) e
[o procedimento de replay](../../docs/cassettes.md).
