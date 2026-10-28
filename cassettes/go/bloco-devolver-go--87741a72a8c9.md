---
type: Cassette
title: bloco-devolver-go
installation: go
source_zip: https://archive.org/download/todos-sei-cassettes-2026/lotes/lote-c7a4af8d9d70ddc3.zip
archive_member: bloco-devolver-go.json
archive_sha1: c7a4af8d9d70ddc3599cdd9bccbb98e6d726db78
cassette_sha256_lf: 87741a72a8c969eec878816af809e322fc1f6f8f41e46d7311718d83a2c975b8
interactions: 74
schema_version: 1
complete: true
---

# bloco-devolver-go

**Instalação:** go (`seitreinamento.go.gov.br`). **Origem:** [ZIP imutável do Internet Archive](https://archive.org/download/todos-sei-cassettes-2026/lotes/lote-c7a4af8d9d70ddc3.zip); membro `bloco-devolver-go.json`.

**Verificações:** SHA-1 do ZIP `c7a4af8d9d70ddc3599cdd9bccbb98e6d726db78`; SHA-256 da cassette após CRLF→LF `87741a72a8c969eec878816af809e322fc1f6f8f41e46d7311718d83a2c975b8`. Commit de captura: `959c0c372f363c2015312465fd5e5c0d2bbf8f35`.

**Observação:** 74 interações HTTP; métodos GET: 49, POST: 25.

## Operações HTTP registradas

- `bloco_assinatura_listar`: 23 requisição(ões)
- `procedimento_visualizar`: 7 requisição(ões)
- `procedimento_controlar`: 5 requisição(ões)
- `rel_bloco_protocolo_listar`: 4 requisição(ões)
- `infra_trocar_unidade`: 4 requisição(ões)
- `procedimento_trabalhar`: 3 requisição(ões)
- `procedimento_escolher_tipo`: 2 requisição(ões)
- `procedimento_gerar`: 2 requisição(ões)
- `documento_escolher_tipo`: 2 requisição(ões)
- `documento_gerar`: 2 requisição(ões)
- `arvore_visualizar`: 2 requisição(ões)
- `bloco_assinatura_cadastrar`: 2 requisição(ões)
- `unidade_auto_completar_outras`: 2 requisição(ões)
- `bloco_escolher`: 2 requisição(ões)
- `protocolo_pesquisa_rapida`: 2 requisição(ões)
- `procedimento_concluir`: 2 requisição(ões)
- `editor_montar`: 1 requisição(ões)
- `editor_salvar_conteudo`: 1 requisição(ões)
- `documento_imprimir_web`: 1 requisição(ões)
- `bloco_assinatura_alterar`: 1 requisição(ões)
- `bloco_disponibilizar`: 1 requisição(ões)
- `rel_bloco_protocolo_excluir`: 1 requisição(ões)
- `bloco_excluir`: 1 requisição(ões)
- `documento_excluir`: 1 requisição(ões)

## Alcance da evidência

A gravação comprova respostas observadas na instalação indicada. Não demonstra,
por si só, paridade com a CLI IES, corpo de requisições POST ou sucesso sem
conferência pós-escrita. Use os passos relevantes e o contrato antes de implementar.

Consulte [como o SEI funciona](../../docs/como-o-sei-funciona/README.md) e
[o procedimento de replay](../../docs/cassettes.md).
