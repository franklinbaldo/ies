---
type: Cassette
title: processo-excluir-ufrj
installation: ufrj
source_zip: https://archive.org/download/todos-sei-cassettes-2026/lotes/lote-c7a4af8d9d70ddc3.zip
archive_member: processo-excluir-ufrj.json
archive_sha1: c7a4af8d9d70ddc3599cdd9bccbb98e6d726db78
cassette_sha256_lf: 30bc65e0e27038280ea027e7e91b102ac596bb59438fa9ebcb0f8d9102f671ff
interactions: 34
schema_version: 1
complete: true
---

# processo-excluir-ufrj

**Instalação:** ufrj (`teste.sei.tic.ufrj.br`). **Origem:** [ZIP imutável do Internet Archive](https://archive.org/download/todos-sei-cassettes-2026/lotes/lote-c7a4af8d9d70ddc3.zip); membro `processo-excluir-ufrj.json`.

**Verificações:** SHA-1 do ZIP `c7a4af8d9d70ddc3599cdd9bccbb98e6d726db78`; SHA-256 da cassette após CRLF→LF `30bc65e0e27038280ea027e7e91b102ac596bb59438fa9ebcb0f8d9102f671ff`. Commit de captura: `b256a352055d2bee1242f8b1ffe5005bbdd401a6`.

**Observação:** 34 interações HTTP; métodos GET: 24, POST: 10.

## Operações HTTP registradas

- `procedimento_visualizar`: 9 requisição(ões)
- `procedimento_escolher_tipo`: 4 requisição(ões)
- `procedimento_gerar`: 4 requisição(ões)
- `procedimento_trabalhar`: 2 requisição(ões)
- `arvore_visualizar`: 2 requisição(ões)
- `protocolo_pesquisa_rapida`: 2 requisição(ões)
- `documento_escolher_tipo`: 2 requisição(ões)
- `documento_gerar`: 2 requisição(ões)
- `procedimento_concluir`: 2 requisição(ões)
- `procedimento_controlar`: 1 requisição(ões)
- `procedimento_excluir`: 1 requisição(ões)
- `editor_montar`: 1 requisição(ões)
- `editor_salvar`: 1 requisição(ões)
- `documento_imprimir_web`: 1 requisição(ões)

## Alcance da evidência

A gravação comprova respostas observadas na instalação indicada. Não demonstra,
por si só, paridade com a CLI IES, corpo de requisições POST ou sucesso sem
conferência pós-escrita. Use os passos relevantes e o contrato antes de implementar.

Consulte [como o SEI funciona](../../docs/como-o-sei-funciona/README.md) e
[o procedimento de replay](../../docs/cassettes.md).
