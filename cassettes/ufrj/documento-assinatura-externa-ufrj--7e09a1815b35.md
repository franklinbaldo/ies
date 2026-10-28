---
type: Cassette
title: documento-assinatura-externa-ufrj
installation: ufrj
source_zip: https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-ufrj.zip
archive_member: documento-assinatura-externa-ufrj.json
archive_sha1: 3b29ba3c4e18fa0b336fbec9490c4f59a15d4aa0
cassette_sha256_lf: 7e09a1815b356dafadcbc706c096edccce060e0ad073e992ea5d208578bd7276
interactions: 41
schema_version: 1
complete: true
---

# documento-assinatura-externa-ufrj

**Instalação:** ufrj (`teste.sei.tic.ufrj.br`). **Origem:** [ZIP imutável do Internet Archive](https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-ufrj.zip); membro `documento-assinatura-externa-ufrj.json`.

**Verificações:** SHA-1 do ZIP `3b29ba3c4e18fa0b336fbec9490c4f59a15d4aa0`; SHA-256 da cassette após CRLF→LF `7e09a1815b356dafadcbc706c096edccce060e0ad073e992ea5d208578bd7276`. Commit de captura: `870ed09550feaa71079b36699d3ec7d038781c95`.

**Observação:** 41 interações HTTP; métodos GET: 31, POST: 10.

## Operações HTTP registradas

- `procedimento_visualizar`: 11 requisição(ões)
- `assinatura_externa_gerenciar`: 9 requisição(ões)
- `procedimento_escolher_tipo`: 2 requisição(ões)
- `procedimento_gerar`: 2 requisição(ões)
- `documento_escolher_tipo`: 2 requisição(ões)
- `documento_gerar`: 2 requisição(ões)
- `usuario_externo_auto_completar`: 2 requisição(ões)
- `assinatura_externa_cancelar`: 2 requisição(ões)
- `procedimento_concluir`: 2 requisição(ões)
- `procedimento_controlar`: 1 requisição(ões)
- `procedimento_trabalhar`: 1 requisição(ões)
- `arvore_visualizar`: 1 requisição(ões)
- `editor_montar`: 1 requisição(ões)
- `editor_salvar`: 1 requisição(ões)
- `documento_imprimir_web`: 1 requisição(ões)
- `assinatura_externa_liberar`: 1 requisição(ões)

## Alcance da evidência

A gravação comprova respostas observadas na instalação indicada. Não demonstra,
por si só, paridade com a CLI IES, corpo de requisições POST ou sucesso sem
conferência pós-escrita. Use os passos relevantes e o contrato antes de implementar.

Consulte [como o SEI funciona](../../docs/como-o-sei-funciona/README.md) e
[o procedimento de replay](../../docs/cassettes.md).
