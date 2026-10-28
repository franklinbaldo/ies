---
type: Cassette
title: documento-enviar-email-go
installation: go
source_zip: https://archive.org/download/todos-sei-cassettes-2026/lotes/lote-a00f2dd917ad3a32.zip
archive_member: documento-enviar-email-go.json
archive_sha1: a00f2dd917ad3a32683c93cc02f0bf0942699e9d
cassette_sha256_lf: b493d15b0b8623832eda031445c424c65a69748b66329f1cb6c322f653faa202
interactions: 43
schema_version: 1
complete: true
---

# documento-enviar-email-go

**Instalação:** go (`seitreinamento.go.gov.br`). **Origem:** [ZIP imutável do Internet Archive](https://archive.org/download/todos-sei-cassettes-2026/lotes/lote-a00f2dd917ad3a32.zip); membro `documento-enviar-email-go.json`.

**Verificações:** SHA-1 do ZIP `a00f2dd917ad3a32683c93cc02f0bf0942699e9d`; SHA-256 da cassette após CRLF→LF `b493d15b0b8623832eda031445c424c65a69748b66329f1cb6c322f653faa202`. Commit de captura: `9592d7531e4417658ba11b6390846f31f472d87c`.

**Observação:** 43 interações HTTP; métodos GET: 34, POST: 9.

## Operações HTTP registradas

- `procedimento_visualizar`: 11 requisição(ões)
- `arvore_visualizar`: 5 requisição(ões)
- `documento_enviar_email`: 5 requisição(ões)
- `documento_imprimir_web`: 3 requisição(ões)
- `procedimento_consultar_historico`: 3 requisição(ões)
- `procedimento_escolher_tipo`: 2 requisição(ões)
- `procedimento_gerar`: 2 requisição(ões)
- `documento_escolher_tipo`: 2 requisição(ões)
- `documento_gerar`: 2 requisição(ões)
- `documento_assinar`: 2 requisição(ões)
- `procedimento_concluir`: 2 requisição(ões)
- `procedimento_controlar`: 1 requisição(ões)
- `procedimento_trabalhar`: 1 requisição(ões)
- `editor_montar`: 1 requisição(ões)
- `editor_salvar_conteudo`: 1 requisição(ões)

## Alcance da evidência

A gravação comprova respostas observadas na instalação indicada. Não demonstra,
por si só, paridade com a CLI IES, corpo de requisições POST ou sucesso sem
conferência pós-escrita. Use os passos relevantes e o contrato antes de implementar.

Consulte [como o SEI funciona](../../docs/como-o-sei-funciona/README.md) e
[o procedimento de replay](../../docs/cassettes.md).
