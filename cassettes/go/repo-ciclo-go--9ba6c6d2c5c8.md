---
type: Cassette
title: repo-ciclo-go
installation: go
source_zip: https://archive.org/download/todos-sei-cassettes-2026/lotes/lote-984713118e55dd08.zip
archive_member: repo-ciclo-go.json
archive_sha1: 984713118e55dd08f7cc82d90c5ab9c844cf6fe2
cassette_sha256_lf: 9ba6c6d2c5c81514a75d316b5963af92cf4970640c2c68f48bf265f9890b7e2d
interactions: 57
schema_version: 1
complete: true
---

# repo-ciclo-go

**Instalação:** go (`seitreinamento.go.gov.br`). **Origem:** [ZIP imutável do Internet Archive](https://archive.org/download/todos-sei-cassettes-2026/lotes/lote-984713118e55dd08.zip); membro `repo-ciclo-go.json`.

**Verificações:** SHA-1 do ZIP `984713118e55dd08f7cc82d90c5ab9c844cf6fe2`; SHA-256 da cassette após CRLF→LF `9ba6c6d2c5c81514a75d316b5963af92cf4970640c2c68f48bf265f9890b7e2d`. Commit de captura: `bdc45e127b06fb501f33c9091583005110c790c0`.

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
