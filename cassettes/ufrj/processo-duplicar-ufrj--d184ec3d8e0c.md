---
type: Cassette
title: processo-duplicar-ufrj
installation: ufrj
source_zip: https://archive.org/download/todos-sei-cassettes-2026/lotes/lote-c7a4af8d9d70ddc3.zip
archive_member: processo-duplicar-ufrj.json
archive_sha1: c7a4af8d9d70ddc3599cdd9bccbb98e6d726db78
cassette_sha256_lf: d184ec3d8e0cf9d12b3057d01c9ef4da7ffa89690017b50db36219a85e1899f5
interactions: 39
schema_version: 1
complete: true
---

# processo-duplicar-ufrj

**Instalação:** ufrj (`teste.sei.tic.ufrj.br`). **Origem:** [ZIP imutável do Internet Archive](https://archive.org/download/todos-sei-cassettes-2026/lotes/lote-c7a4af8d9d70ddc3.zip); membro `processo-duplicar-ufrj.json`.

**Verificações:** SHA-1 do ZIP `c7a4af8d9d70ddc3599cdd9bccbb98e6d726db78`; SHA-256 da cassette após CRLF→LF `d184ec3d8e0cf9d12b3057d01c9ef4da7ffa89690017b50db36219a85e1899f5`. Commit de captura: `fa1e6f20d41261ede1d291facc0f8165c8298ab2`.

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
- `editor_salvar`: 2 requisição(ões)
- `documento_imprimir_web`: 2 requisição(ões)
- `procedimento_controlar`: 1 requisição(ões)

## Alcance da evidência

A gravação comprova respostas observadas na instalação indicada. Não demonstra,
por si só, paridade com a CLI IES, corpo de requisições POST ou sucesso sem
conferência pós-escrita. Use os passos relevantes e o contrato antes de implementar.

Consulte [como o SEI funciona](../../docs/como-o-sei-funciona/README.md) e
[o procedimento de replay](../../docs/cassettes.md).
