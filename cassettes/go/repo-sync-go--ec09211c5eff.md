---
type: Cassette
title: repo-sync-go
installation: go
source_zip: https://archive.org/download/todos-sei-cassettes-2026/lotes/lote-b26c2e66c244ad0a.zip
archive_member: repo-sync-go.json
archive_sha1: b26c2e66c244ad0ac278cdec8bbd47bf901b352c
cassette_sha256_lf: ec09211c5eff048229eb2b6990b58add0d830d7e51f2cdddb5720639e34aafcb
interactions: 94
schema_version: 1
complete: true
---

# repo-sync-go

**Instalação:** go (`seitreinamento.go.gov.br`). **Origem:** [ZIP imutável do Internet Archive](https://archive.org/download/todos-sei-cassettes-2026/lotes/lote-b26c2e66c244ad0a.zip); membro `repo-sync-go.json`.

**Verificações:** SHA-1 do ZIP `b26c2e66c244ad0ac278cdec8bbd47bf901b352c`; SHA-256 da cassette após CRLF→LF `ec09211c5eff048229eb2b6990b58add0d830d7e51f2cdddb5720639e34aafcb`. Commit de captura: `93fd4de7a4225afc9ac03e06566e70369adcff4e`.

**Observação:** 94 interações HTTP; métodos GET: 48, POST: 46.

## Operações HTTP registradas

- `texto_padrao_interno_listar`: 67 requisição(ões)
- `texto_padrao_interno_consultar`: 10 requisição(ões)
- `infra_trocar_unidade`: 9 requisição(ões)
- `texto_padrao_interno_alterar`: 4 requisição(ões)
- `texto_padrao_interno_cadastrar`: 2 requisição(ões)
- `procedimento_controlar`: 1 requisição(ões)
- `texto_padrao_interno_excluir`: 1 requisição(ões)

## Alcance da evidência

A gravação comprova respostas observadas na instalação indicada. Não demonstra,
por si só, paridade com a CLI IES, corpo de requisições POST ou sucesso sem
conferência pós-escrita. Use os passos relevantes e o contrato antes de implementar.

Consulte [como o SEI funciona](../../docs/como-o-sei-funciona/README.md) e
[o procedimento de replay](../../docs/cassettes.md).
