---
type: Cassette
title: processo-excluir-unb
installation: unb
source_zip: https://archive.org/download/todos-sei-cassettes-2026/lotes/lote-c7a4af8d9d70ddc3.zip
archive_member: processo-excluir-unb.json
archive_sha1: c7a4af8d9d70ddc3599cdd9bccbb98e6d726db78
cassette_sha256_lf: 160fadda318c0fbc7773cba9978f8b4889f2900e07ccdc6970b465c1db2b9f56
interactions: 32
schema_version: 1
complete: true
---

# processo-excluir-unb

**Instalação:** unb (`sei.treinamento.unb.br`). **Origem:** [ZIP imutável do Internet Archive](https://archive.org/download/todos-sei-cassettes-2026/lotes/lote-c7a4af8d9d70ddc3.zip); membro `processo-excluir-unb.json`.

**Verificações:** SHA-1 do ZIP `c7a4af8d9d70ddc3599cdd9bccbb98e6d726db78`; SHA-256 da cassette após CRLF→LF `160fadda318c0fbc7773cba9978f8b4889f2900e07ccdc6970b465c1db2b9f56`. Commit de captura: `3c0857f11635fb77e22aa23996e4d36251831e86`.

**Observação:** 32 interações HTTP; métodos GET: 26, POST: 6.

## Operações HTTP registradas

- `procedimento_visualizar`: 9 requisição(ões)
- `procedimento_gerar`: 4 requisição(ões)
- `arvore_visualizar`: 3 requisição(ões)
- `procedimento_escolher_tipo`: 2 requisição(ões)
- `procedimento_trabalhar`: 2 requisição(ões)
- `protocolo_pesquisa_rapida`: 2 requisição(ões)
- `documento_gerar`: 2 requisição(ões)
- `procedimento_controlar`: 1 requisição(ões)
- `procedimento_excluir`: 1 requisição(ões)
- `documento_escolher_tipo`: 1 requisição(ões)
- `editor_montar`: 1 requisição(ões)
- `editor_salvar`: 1 requisição(ões)
- `documento_imprimir_web`: 1 requisição(ões)
- `documento_visualizar`: 1 requisição(ões)
- `procedimento_concluir`: 1 requisição(ões)

## Alcance da evidência

A gravação comprova respostas observadas na instalação indicada. Não demonstra,
por si só, paridade com a CLI IES, corpo de requisições POST ou sucesso sem
conferência pós-escrita. Use os passos relevantes e o contrato antes de implementar.

Consulte [como o SEI funciona](../../docs/como-o-sei-funciona/README.md) e
[o procedimento de replay](../../docs/cassettes.md).
