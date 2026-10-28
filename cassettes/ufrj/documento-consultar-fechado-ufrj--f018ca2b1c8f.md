---
type: Cassette
title: documento-consultar-fechado-ufrj
installation: ufrj
source_zip: https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-ufrj.zip
archive_member: documento-consultar-fechado-ufrj.json
archive_sha1: 3b29ba3c4e18fa0b336fbec9490c4f59a15d4aa0
cassette_sha256_lf: f018ca2b1c8fcd33f2d24f4a53c0ec0e67de677e4b71635921beb64ad026a5fd
interactions: 45
schema_version: 1
complete: true
---

# documento-consultar-fechado-ufrj

**Instalação:** ufrj (`teste.sei.tic.ufrj.br`). **Origem:** [ZIP imutável do Internet Archive](https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-ufrj.zip); membro `documento-consultar-fechado-ufrj.json`.

**Verificações:** SHA-1 do ZIP `3b29ba3c4e18fa0b336fbec9490c4f59a15d4aa0`; SHA-256 da cassette após CRLF→LF `f018ca2b1c8fcd33f2d24f4a53c0ec0e67de677e4b71635921beb64ad026a5fd`. Commit de captura: `4a1b7942b8ac8697801ea8eb21d81b6f7c81c439`.

**Observação:** 45 interações HTTP; métodos GET: 30, POST: 15.

## Operações HTTP registradas

- `procedimento_visualizar`: 8 requisição(ões)
- `documento_escolher_tipo`: 4 requisição(ões)
- `procedimento_escolher_tipo`: 3 requisição(ões)
- `documento_imprimir_web`: 3 requisição(ões)
- `documento_receber`: 3 requisição(ões)
- `procedimento_gerar`: 2 requisição(ões)
- `documento_gerar`: 2 requisição(ões)
- `arvore_visualizar`: 2 requisição(ões)
- `documento_assinar`: 2 requisição(ões)
- `documento_recebido_duplicado`: 2 requisição(ões)
- `procedimento_concluir`: 2 requisição(ões)
- `procedimento_gerar_zip`: 2 requisição(ões)
- `procedimento_controlar`: 1 requisição(ões)
- `procedimento_trabalhar`: 1 requisição(ões)
- `editor_montar`: 1 requisição(ões)
- `editor_salvar`: 1 requisição(ões)
- `documento_upload_anexo`: 1 requisição(ões)
- `procedimento_consultar`: 1 requisição(ões)
- `anotacao_registrar`: 1 requisição(ões)
- `exibir_arquivo`: 1 requisição(ões)
- `documento_consultar`: 1 requisição(ões)
- `documento_consultar_recebido`: 1 requisição(ões)

## Alcance da evidência

A gravação comprova respostas observadas na instalação indicada. Não demonstra,
por si só, paridade com a CLI IES, corpo de requisições POST ou sucesso sem
conferência pós-escrita. Use os passos relevantes e o contrato antes de implementar.

Consulte [como o SEI funciona](../../docs/como-o-sei-funciona/README.md) e
[o procedimento de replay](../../docs/cassettes.md).
