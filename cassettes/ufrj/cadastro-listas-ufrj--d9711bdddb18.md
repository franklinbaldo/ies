---
type: Cassette
title: cadastro-listas-ufrj
installation: ufrj
source_zip: https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-ufrj.zip
archive_member: cadastro-listas-ufrj.json
archive_sha1: 3b29ba3c4e18fa0b336fbec9490c4f59a15d4aa0
cassette_sha256_lf: d9711bdddb189cb9f18f895aac042305b2f418c175df3e5643b298d2999efbea
interactions: 19
schema_version: 1
complete: true
---

# cadastro-listas-ufrj

**Instalação:** ufrj (`teste.sei.tic.ufrj.br`). **Origem:** [ZIP imutável do Internet Archive](https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-ufrj.zip); membro `cadastro-listas-ufrj.json`.

**Verificações:** SHA-1 do ZIP `3b29ba3c4e18fa0b336fbec9490c4f59a15d4aa0`; SHA-256 da cassette após CRLF→LF `d9711bdddb189cb9f18f895aac042305b2f418c175df3e5643b298d2999efbea`. Commit de captura: `3613f8b3ab4f1aa2b1ac065b3748961f1b2f20dc`.

**Observação:** 19 interações HTTP; métodos GET: 12, POST: 7.

## Operações HTTP registradas

- `documento_escolher_tipo`: 6 requisição(ões)
- `procedimento_escolher_tipo`: 4 requisição(ões)
- `documento_receber`: 2 requisição(ões)
- `procedimento_controlar`: 1 requisição(ões)
- `procedimento_gerar`: 1 requisição(ões)
- `hipotese_legal_select_nome_base_legal`: 1 requisição(ões)
- `protocolo_pesquisar`: 1 requisição(ões)
- `protocolo_pesquisa_rapida`: 1 requisição(ões)
- `procedimento_trabalhar`: 1 requisição(ões)
- `procedimento_visualizar`: 1 requisição(ões)

## Alcance da evidência

A gravação comprova respostas observadas na instalação indicada. Não demonstra,
por si só, paridade com a CLI IES, corpo de requisições POST ou sucesso sem
conferência pós-escrita. Use os passos relevantes e o contrato antes de implementar.

Consulte [como o SEI funciona](../../docs/como-o-sei-funciona/README.md) e
[o procedimento de replay](../../docs/cassettes.md).
