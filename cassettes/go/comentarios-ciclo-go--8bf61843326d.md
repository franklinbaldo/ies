---
type: Cassette
title: comentarios-ciclo-go
installation: go
source_zip: https://archive.org/download/todos-sei-cassettes-2026/lotes/lote-4135281e5da127f1.zip
archive_member: comentarios-ciclo-go.json
archive_sha1: 4135281e5da127f1a93ccdd6193b742b930bf7e7
cassette_sha256_lf: 8bf61843326d8346754953dddc7e511c71474116754f4a60210c2cc785b1a273
interactions: 39
schema_version: 1
complete: true
---

# comentarios-ciclo-go

**Instalação:** go (`seitreinamento.go.gov.br`). **Origem:** [ZIP imutável do Internet Archive](https://archive.org/download/todos-sei-cassettes-2026/lotes/lote-4135281e5da127f1.zip); membro `comentarios-ciclo-go.json`.

**Verificações:** SHA-1 do ZIP `4135281e5da127f1a93ccdd6193b742b930bf7e7`; SHA-256 da cassette após CRLF→LF `8bf61843326d8346754953dddc7e511c71474116754f4a60210c2cc785b1a273`. Commit de captura: `82a2cdb6eaa65dec2f7b8fce4ecfb88c0a7ef600`.

**Observação:** 39 interações HTTP; métodos GET: 29, POST: 10.

## Operações HTTP registradas

- `comentario_listar`: 11 requisição(ões)
- `procedimento_visualizar`: 6 requisição(ões)
- `comentario_cadastrar`: 6 requisição(ões)
- `procedimento_escolher_tipo`: 2 requisição(ões)
- `procedimento_gerar`: 2 requisição(ões)
- `documento_escolher_tipo`: 2 requisição(ões)
- `documento_gerar`: 2 requisição(ões)
- `procedimento_concluir`: 2 requisição(ões)
- `procedimento_controlar`: 1 requisição(ões)
- `procedimento_trabalhar`: 1 requisição(ões)
- `arvore_visualizar`: 1 requisição(ões)
- `editor_montar`: 1 requisição(ões)
- `editor_salvar_conteudo`: 1 requisição(ões)
- `documento_imprimir_web`: 1 requisição(ões)

## Alcance da evidência

A gravação comprova respostas observadas na instalação indicada. Não demonstra,
por si só, paridade com a CLI IES, corpo de requisições POST ou sucesso sem
conferência pós-escrita. Use os passos relevantes e o contrato antes de implementar.

Consulte [como o SEI funciona](../../docs/como-o-sei-funciona/README.md) e
[o procedimento de replay](../../docs/cassettes.md).
