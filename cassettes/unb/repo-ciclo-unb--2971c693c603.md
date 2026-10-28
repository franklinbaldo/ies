---
type: Cassette
title: repo-ciclo-unb
installation: unb
source_zip: https://archive.org/download/todos-sei-cassettes-2026/lotes/lote-984713118e55dd08.zip
archive_member: repo-ciclo-unb.json
archive_sha1: 984713118e55dd08f7cc82d90c5ab9c844cf6fe2
cassette_sha256_lf: 2971c693c603b68995fb5e23e86c2a48ae4d05af055814b3ff09bf716bbd33db
interactions: 65
schema_version: 1
complete: true
---

# repo-ciclo-unb

**Instalação:** unb (`sei.treinamento.unb.br`). **Origem:** [ZIP imutável do Internet Archive](https://archive.org/download/todos-sei-cassettes-2026/lotes/lote-984713118e55dd08.zip); membro `repo-ciclo-unb.json`.

**Verificações:** SHA-1 do ZIP `984713118e55dd08f7cc82d90c5ab9c844cf6fe2`; SHA-256 da cassette após CRLF→LF `2971c693c603b68995fb5e23e86c2a48ae4d05af055814b3ff09bf716bbd33db`. Commit de captura: `497a359abd5317fb555c7cd8c3651463150efe65`.

**Observação:** 65 interações HTTP; métodos GET: 33, POST: 32.

## Operações HTTP registradas

- `texto_padrao_interno_listar`: 48 requisição(ões)
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
