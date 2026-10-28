---
type: Cassette
title: repo-push-go
installation: go
source_zip: https://archive.org/download/todos-sei-cassettes-2026/lotes/lote-984713118e55dd08.zip
archive_member: repo-push-go.json
archive_sha1: 984713118e55dd08f7cc82d90c5ab9c844cf6fe2
cassette_sha256_lf: ef2f56fb258e1e744c2258db47315c568e82bc3a277e013c7c52d372a1845dcb
interactions: 81
schema_version: 1
complete: true
---

# repo-push-go

**Instalação:** go (`seitreinamento.go.gov.br`). **Origem:** [ZIP imutável do Internet Archive](https://archive.org/download/todos-sei-cassettes-2026/lotes/lote-984713118e55dd08.zip); membro `repo-push-go.json`.

**Verificações:** SHA-1 do ZIP `984713118e55dd08f7cc82d90c5ab9c844cf6fe2`; SHA-256 da cassette após CRLF→LF `ef2f56fb258e1e744c2258db47315c568e82bc3a277e013c7c52d372a1845dcb`. Commit de captura: `8283da51fd17db2cdafe7daba207a3d36d3b38d6`.

**Observação:** 81 interações HTTP; métodos GET: 42, POST: 39.

## Operações HTTP registradas

- `texto_padrao_interno_listar`: 58 requisição(ões)
- `infra_trocar_unidade`: 8 requisição(ões)
- `texto_padrao_interno_consultar`: 8 requisição(ões)
- `texto_padrao_interno_alterar`: 3 requisição(ões)
- `texto_padrao_interno_cadastrar`: 2 requisição(ões)
- `procedimento_controlar`: 1 requisição(ões)
- `texto_padrao_interno_excluir`: 1 requisição(ões)

## Alcance da evidência

A gravação comprova respostas observadas na instalação indicada. Não demonstra,
por si só, paridade com a CLI IES, corpo de requisições POST ou sucesso sem
conferência pós-escrita. Use os passos relevantes e o contrato antes de implementar.

Consulte [como o SEI funciona](../../docs/como-o-sei-funciona/README.md) e
[o procedimento de replay](../../docs/cassettes.md).
