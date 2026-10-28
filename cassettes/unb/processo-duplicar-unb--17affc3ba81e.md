---
type: Cassette
title: processo-duplicar-unb
installation: unb
source_zip: https://archive.org/download/todos-sei-cassettes-2026/lotes/lote-c7a4af8d9d70ddc3.zip
archive_member: processo-duplicar-unb.json
archive_sha1: c7a4af8d9d70ddc3599cdd9bccbb98e6d726db78
cassette_sha256_lf: 17affc3ba81e2f0060b2f392d8433762123c57588fb4d065503c9f876d10d6a0
interactions: 38
schema_version: 1
complete: true
---

# processo-duplicar-unb

**Instalação:** unb (`sei.treinamento.unb.br`). **Origem:** [ZIP imutável do Internet Archive](https://archive.org/download/todos-sei-cassettes-2026/lotes/lote-c7a4af8d9d70ddc3.zip); membro `processo-duplicar-unb.json`.

**Verificações:** SHA-1 do ZIP `c7a4af8d9d70ddc3599cdd9bccbb98e6d726db78`; SHA-256 da cassette após CRLF→LF `17affc3ba81e2f0060b2f392d8433762123c57588fb4d065503c9f876d10d6a0`. Commit de captura: `be65cefd6ac4320cbda93a79656c21bb7c639d64`.

**Observação:** 38 interações HTTP; métodos GET: 32, POST: 6.

## Operações HTTP registradas

- `procedimento_visualizar`: 8 requisição(ões)
- `documento_gerar`: 4 requisição(ões)
- `arvore_visualizar`: 4 requisição(ões)
- `procedimento_duplicar`: 4 requisição(ões)
- `procedimento_gerar`: 2 requisição(ões)
- `procedimento_trabalhar`: 2 requisição(ões)
- `documento_escolher_tipo`: 2 requisição(ões)
- `editor_montar`: 2 requisição(ões)
- `editor_salvar`: 2 requisição(ões)
- `documento_imprimir_web`: 2 requisição(ões)
- `documento_visualizar`: 2 requisição(ões)
- `procedimento_concluir`: 2 requisição(ões)
- `procedimento_controlar`: 1 requisição(ões)
- `procedimento_escolher_tipo`: 1 requisição(ões)

## Alcance da evidência

A gravação comprova respostas observadas na instalação indicada. Não demonstra,
por si só, paridade com a CLI IES, corpo de requisições POST ou sucesso sem
conferência pós-escrita. Use os passos relevantes e o contrato antes de implementar.

Consulte [como o SEI funciona](../../docs/como-o-sei-funciona/README.md) e
[o procedimento de replay](../../docs/cassettes.md).
