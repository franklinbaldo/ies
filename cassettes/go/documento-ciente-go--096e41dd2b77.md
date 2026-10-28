---
type: Cassette
title: documento-ciente-go
installation: go
source_zip: https://archive.org/download/todos-sei-cassettes-2026/lotes/lote-c7a4af8d9d70ddc3.zip
archive_member: documento-ciente-go.json
archive_sha1: c7a4af8d9d70ddc3599cdd9bccbb98e6d726db78
cassette_sha256_lf: 096e41dd2b770de0f1425bb6ffebb257a3feb184b64b520d3cebb91ab1723f80
interactions: 38
schema_version: 1
complete: true
---

# documento-ciente-go

**Instalação:** go (`seitreinamento.go.gov.br`). **Origem:** [ZIP imutável do Internet Archive](https://archive.org/download/todos-sei-cassettes-2026/lotes/lote-c7a4af8d9d70ddc3.zip); membro `documento-ciente-go.json`.

**Verificações:** SHA-1 do ZIP `c7a4af8d9d70ddc3599cdd9bccbb98e6d726db78`; SHA-256 da cassette após CRLF→LF `096e41dd2b770de0f1425bb6ffebb257a3feb184b64b520d3cebb91ab1723f80`. Commit de captura: `5e070cfd34612b46dd8451a6fe6e5d2f3b130beb`.

**Observação:** 38 interações HTTP; métodos GET: 30, POST: 8.

## Operações HTTP registradas

- `procedimento_visualizar`: 10 requisição(ões)
- `procedimento_escolher_tipo`: 4 requisição(ões)
- `procedimento_gerar`: 3 requisição(ões)
- `arvore_visualizar`: 3 requisição(ões)
- `documento_imprimir_web`: 3 requisição(ões)
- `documento_escolher_tipo`: 2 requisição(ões)
- `documento_gerar`: 2 requisição(ões)
- `documento_assinar`: 2 requisição(ões)
- `documento_ciencia`: 2 requisição(ões)
- `procedimento_concluir`: 2 requisição(ões)
- `procedimento_controlar`: 1 requisição(ões)
- `procedimento_trabalhar`: 1 requisição(ões)
- `editor_montar`: 1 requisição(ões)
- `editor_salvar_conteudo`: 1 requisição(ões)
- `protocolo_ciencia_listar`: 1 requisição(ões)

## Alcance da evidência

A gravação comprova respostas observadas na instalação indicada. Não demonstra,
por si só, paridade com a CLI IES, corpo de requisições POST ou sucesso sem
conferência pós-escrita. Use os passos relevantes e o contrato antes de implementar.

Consulte [como o SEI funciona](../../docs/como-o-sei-funciona/README.md) e
[o procedimento de replay](../../docs/cassettes.md).
