---
type: Cassette
title: processo-acompanhar-go
installation: go
source_zip: https://archive.org/download/todos-sei-cassettes-2026/lotes/lote-c7a4af8d9d70ddc3.zip
archive_member: processo-acompanhar-go.json
archive_sha1: c7a4af8d9d70ddc3599cdd9bccbb98e6d726db78
cassette_sha256_lf: d8a021f54e68d52cfa3efa637c6f75b93df1f296c54d464570278efeb7c5815e
interactions: 47
schema_version: 1
complete: true
---

# processo-acompanhar-go

**Instalação:** go (`seitreinamento.go.gov.br`). **Origem:** [ZIP imutável do Internet Archive](https://archive.org/download/todos-sei-cassettes-2026/lotes/lote-c7a4af8d9d70ddc3.zip); membro `processo-acompanhar-go.json`.

**Verificações:** SHA-1 do ZIP `c7a4af8d9d70ddc3599cdd9bccbb98e6d726db78`; SHA-256 da cassette após CRLF→LF `d8a021f54e68d52cfa3efa637c6f75b93df1f296c54d464570278efeb7c5815e`. Commit de captura: `f7dbb5597d25a18267997ba85ccd8e22513a43f2`.

**Observação:** 47 interações HTTP; métodos GET: 38, POST: 9.

## Operações HTTP registradas

- `acompanhamento_gerenciar`: 13 requisição(ões)
- `procedimento_visualizar`: 12 requisição(ões)
- `acompanhamento_cadastrar`: 9 requisição(ões)
- `procedimento_escolher_tipo`: 4 requisição(ões)
- `procedimento_gerar`: 3 requisição(ões)
- `acompanhamento_excluir`: 2 requisição(ões)
- `procedimento_concluir`: 2 requisição(ões)
- `procedimento_controlar`: 1 requisição(ões)
- `procedimento_trabalhar`: 1 requisição(ões)

## Alcance da evidência

A gravação comprova respostas observadas na instalação indicada. Não demonstra,
por si só, paridade com a CLI IES, corpo de requisições POST ou sucesso sem
conferência pós-escrita. Use os passos relevantes e o contrato antes de implementar.

Consulte [como o SEI funciona](../../docs/como-o-sei-funciona/README.md) e
[o procedimento de replay](../../docs/cassettes.md).
