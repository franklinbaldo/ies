---
type: Cassette
title: repo-ciclo-go
installation: go
source_zip: https://archive.org/download/todos-sei-cassettes-2026/lotes/lote-eb4d07a9c441a2ef.zip
archive_member: repo-ciclo-go.json
archive_sha1: eb4d07a9c441a2ef9c28c146aa0d59d9a78a0eda
cassette_sha256_lf: f87e8127672e3d2a1c300620fba7f92b82ce3806b914881592d6741b130277bc
interactions: 57
schema_version: 1
complete: true
---

# repo-ciclo-go

**Instalação:** go (`seitreinamento.go.gov.br`). **Origem:** [ZIP imutável do Internet Archive](https://archive.org/download/todos-sei-cassettes-2026/lotes/lote-eb4d07a9c441a2ef.zip); membro `repo-ciclo-go.json`.

**Verificações:** SHA-1 do ZIP `eb4d07a9c441a2ef9c28c146aa0d59d9a78a0eda`; SHA-256 da cassette após CRLF→LF `f87e8127672e3d2a1c300620fba7f92b82ce3806b914881592d6741b130277bc`. Commit de captura: `66a7cdfa8ada0f2790278cc58766cc5a0075bb05`.

**Observação:** 57 interações HTTP; métodos GET: 33, POST: 24.

## Operações HTTP registradas

- `texto_padrao_interno_listar`: 40 requisição(ões)
- `infra_trocar_unidade`: 9 requisição(ões)
- `texto_padrao_interno_cadastrar`: 3 requisição(ões)
- `texto_padrao_interno_consultar`: 3 requisição(ões)
- `procedimento_controlar`: 1 requisição(ões)
- `texto_padrao_interno_excluir`: 1 requisição(ões)

## Alcance da evidência

A gravação comprova respostas observadas na instalação indicada. Não demonstra,
por si só, paridade com a CLI IES, corpo de requisições POST ou sucesso sem
conferência pós-escrita. Use os passos relevantes e o contrato antes de implementar.

Consulte [como o SEI funciona](../../docs/como-o-sei-funciona/README.md) e
[o procedimento de replay](../../docs/cassettes.md).
