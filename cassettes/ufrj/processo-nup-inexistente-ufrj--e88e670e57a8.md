---
type: Cassette
title: processo-nup-inexistente-ufrj
installation: ufrj
source_zip: https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-ufrj.zip
archive_member: processo-nup-inexistente-ufrj.json
archive_sha1: 3b29ba3c4e18fa0b336fbec9490c4f59a15d4aa0
cassette_sha256_lf: e88e670e57a81806f6e655f1f0b130e29c917dd91369c004cf097b351c76e232
interactions: 12
schema_version: 1
complete: true
---

# processo-nup-inexistente-ufrj

**Instalação:** ufrj (`teste.sei.tic.ufrj.br`). **Origem:** [ZIP imutável do Internet Archive](https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-ufrj.zip); membro `processo-nup-inexistente-ufrj.json`.

**Verificações:** SHA-1 do ZIP `3b29ba3c4e18fa0b336fbec9490c4f59a15d4aa0`; SHA-256 da cassette após CRLF→LF `e88e670e57a81806f6e655f1f0b130e29c917dd91369c004cf097b351c76e232`. Commit de captura: `0d061f58b5833d793827738252b703eb9614708f`.

**Observação:** 12 interações HTTP; métodos GET: 11, POST: 1.

## Operações HTTP registradas

- `procedimento_trabalhar`: 2 requisição(ões)
- `procedimento_controlar`: 1 requisição(ões)
- `protocolo_pesquisa_rapida`: 1 requisição(ões)
- `procedimento_visualizar`: 1 requisição(ões)
- `andamento_marcador_gerenciar`: 1 requisição(ões)
- `andamento_marcador_cadastrar`: 1 requisição(ões)
- `procedimento_alterar`: 1 requisição(ões)
- `anotacao_registrar`: 1 requisição(ões)
- `procedimento_atribuicao_cadastrar`: 1 requisição(ões)
- `procedimento_relacionar`: 1 requisição(ões)
- `procedimento_consultar_historico`: 1 requisição(ões)

## Alcance da evidência

A gravação comprova respostas observadas na instalação indicada. Não demonstra,
por si só, paridade com a CLI IES, corpo de requisições POST ou sucesso sem
conferência pós-escrita. Use os passos relevantes e o contrato antes de implementar.

Consulte [como o SEI funciona](../../docs/como-o-sei-funciona/README.md) e
[o procedimento de replay](../../docs/cassettes.md).
