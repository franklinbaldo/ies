---
type: Cassette
title: processo-duplicar-go
installation: go
source_zip: https://archive.org/download/todos-sei-cassettes-2026/lotes/lote-a00f2dd917ad3a32.zip
archive_member: processo-duplicar-go.json
archive_sha1: a00f2dd917ad3a32683c93cc02f0bf0942699e9d
cassette_sha256_lf: 624c6d290205e224935bd1e983158b390c645a3908ddeffedcec840eccc00208
interactions: 39
schema_version: 1
complete: true
---

# processo-duplicar-go

**Instalação:** go (`seitreinamento.go.gov.br`). **Origem:** [ZIP imutável do Internet Archive](https://archive.org/download/todos-sei-cassettes-2026/lotes/lote-a00f2dd917ad3a32.zip); membro `processo-duplicar-go.json`.

**Verificações:** SHA-1 do ZIP `a00f2dd917ad3a32683c93cc02f0bf0942699e9d`; SHA-256 da cassette após CRLF→LF `624c6d290205e224935bd1e983158b390c645a3908ddeffedcec840eccc00208`. Commit de captura: `96032cba5a45b9cd73bf2f8a56e29829c5a8be77`.

**Observação:** 39 interações HTTP; métodos GET: 28, POST: 11.

## Operações HTTP registradas

- `procedimento_visualizar`: 8 requisição(ões)
- `documento_escolher_tipo`: 4 requisição(ões)
- `documento_gerar`: 4 requisição(ões)
- `procedimento_duplicar`: 4 requisição(ões)
- `procedimento_concluir`: 4 requisição(ões)
- `procedimento_escolher_tipo`: 2 requisição(ões)
- `procedimento_gerar`: 2 requisição(ões)
- `procedimento_trabalhar`: 2 requisição(ões)
- `arvore_visualizar`: 2 requisição(ões)
- `editor_montar`: 2 requisição(ões)
- `editor_salvar_conteudo`: 2 requisição(ões)
- `documento_imprimir_web`: 2 requisição(ões)
- `procedimento_controlar`: 1 requisição(ões)

## Alcance da evidência

A gravação comprova respostas observadas na instalação indicada. Não demonstra,
por si só, paridade com a CLI IES, corpo de requisições POST ou sucesso sem
conferência pós-escrita. Use os passos relevantes e o contrato antes de implementar.

Consulte [como o SEI funciona](../../docs/como-o-sei-funciona/README.md) e
[o procedimento de replay](../../docs/cassettes.md).
