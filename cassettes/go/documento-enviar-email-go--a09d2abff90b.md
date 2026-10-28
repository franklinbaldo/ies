---
type: Cassette
title: documento-enviar-email-go
installation: go
source_zip: https://archive.org/download/todos-sei-cassettes-2026/lotes/lote-c7a4af8d9d70ddc3.zip
archive_member: documento-enviar-email-go.json
archive_sha1: c7a4af8d9d70ddc3599cdd9bccbb98e6d726db78
cassette_sha256_lf: a09d2abff90b32210d8d6f1eeb30fea9c74714a2f9cbfded1b1d3cd315090c19
interactions: 32
schema_version: 1
complete: true
---

# documento-enviar-email-go

**Instalação:** go (`seitreinamento.go.gov.br`). **Origem:** [ZIP imutável do Internet Archive](https://archive.org/download/todos-sei-cassettes-2026/lotes/lote-c7a4af8d9d70ddc3.zip); membro `documento-enviar-email-go.json`.

**Verificações:** SHA-1 do ZIP `c7a4af8d9d70ddc3599cdd9bccbb98e6d726db78`; SHA-256 da cassette após CRLF→LF `a09d2abff90b32210d8d6f1eeb30fea9c74714a2f9cbfded1b1d3cd315090c19`. Commit de captura: `998f6acadd1bca1a11df6a9a9c316eb715a4b2b2`.

**Observação:** 32 interações HTTP; métodos GET: 25, POST: 7.

## Operações HTTP registradas

- `procedimento_visualizar`: 8 requisição(ões)
- `arvore_visualizar`: 5 requisição(ões)
- `documento_imprimir_web`: 3 requisição(ões)
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
