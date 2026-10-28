---
type: Cassette
title: documento-publicacao-ufrj
installation: ufrj
source_zip: https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-ufrj.zip
archive_member: documento-publicacao-ufrj.json
archive_sha1: 3b29ba3c4e18fa0b336fbec9490c4f59a15d4aa0
cassette_sha256_lf: 587bb805b65efa838467972586421dfbe0602a385569e046ed0edbff28c5af63
interactions: 43
schema_version: 1
complete: true
---

# documento-publicacao-ufrj

**Instalação:** ufrj (`teste.sei.tic.ufrj.br`). **Origem:** [ZIP imutável do Internet Archive](https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-ufrj.zip); membro `documento-publicacao-ufrj.json`.

**Verificações:** SHA-1 do ZIP `3b29ba3c4e18fa0b336fbec9490c4f59a15d4aa0`; SHA-256 da cassette após CRLF→LF `587bb805b65efa838467972586421dfbe0602a385569e046ed0edbff28c5af63`. Commit de captura: `ec494c97621e66be1b54b5549df05feec7406d00`.

**Observação:** 43 interações HTTP; métodos GET: 34, POST: 9.

## Operações HTTP registradas

- `procedimento_visualizar`: 14 requisição(ões)
- `publicacao_listar`: 4 requisição(ões)
- `documento_imprimir_web`: 3 requisição(ões)
- `publicacao_agendar`: 3 requisição(ões)
- `procedimento_escolher_tipo`: 2 requisição(ões)
- `procedimento_gerar`: 2 requisição(ões)
- `documento_escolher_tipo`: 2 requisição(ões)
- `documento_gerar`: 2 requisição(ões)
- `arvore_visualizar`: 2 requisição(ões)
- `documento_assinar`: 2 requisição(ões)
- `procedimento_concluir`: 2 requisição(ões)
- `procedimento_controlar`: 1 requisição(ões)
- `procedimento_trabalhar`: 1 requisição(ões)
- `editor_montar`: 1 requisição(ões)
- `editor_salvar`: 1 requisição(ões)
- `publicacao_cancelar_agendamento`: 1 requisição(ões)

## Alcance da evidência

A gravação comprova respostas observadas na instalação indicada. Não demonstra,
por si só, paridade com a CLI IES, corpo de requisições POST ou sucesso sem
conferência pós-escrita. Use os passos relevantes e o contrato antes de implementar.

Consulte [como o SEI funciona](../../docs/como-o-sei-funciona/README.md) e
[o procedimento de replay](../../docs/cassettes.md).
