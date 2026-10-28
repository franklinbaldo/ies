---
type: Cassette
title: repo-sync-unb
installation: unb
source_zip: https://archive.org/download/todos-sei-cassettes-2026/lotes/lote-b26c2e66c244ad0a.zip
archive_member: repo-sync-unb.json
archive_sha1: b26c2e66c244ad0ac278cdec8bbd47bf901b352c
cassette_sha256_lf: 489192a5345f2ffd74a55aa3cd96a22dec3a659405559e71c22a1414b67ac678
interactions: 98
schema_version: 1
complete: true
---

# repo-sync-unb

**Instalação:** unb (`sei.treinamento.unb.br`). **Origem:** [ZIP imutável do Internet Archive](https://archive.org/download/todos-sei-cassettes-2026/lotes/lote-b26c2e66c244ad0a.zip); membro `repo-sync-unb.json`.

**Verificações:** SHA-1 do ZIP `b26c2e66c244ad0ac278cdec8bbd47bf901b352c`; SHA-256 da cassette após CRLF→LF `489192a5345f2ffd74a55aa3cd96a22dec3a659405559e71c22a1414b67ac678`. Commit de captura: `e9533f65ae7355e757f3a6ca56b83a3d403babc2`.

**Observação:** 98 interações HTTP; métodos GET: 48, POST: 50.

## Operações HTTP registradas

- `texto_padrao_interno_listar`: 71 requisição(ões)
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
