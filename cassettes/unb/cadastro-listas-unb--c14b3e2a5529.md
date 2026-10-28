---
type: Cassette
title: cadastro-listas-unb
installation: unb
source_zip: https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-unb.zip
archive_member: cadastro-listas-unb.json
archive_sha1: 21e0559015c56c403daf8933e5040504a7b83837
cassette_sha256_lf: c14b3e2a55295f9a4e88b4e04d4fe503e0ad3eafdf28b6eb9abe15860ec100e5
interactions: 16
schema_version: 1
complete: true
---

# cadastro-listas-unb

**Instalação:** unb (`sei.treinamento.unb.br`). **Origem:** [ZIP imutável do Internet Archive](https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-unb.zip); membro `cadastro-listas-unb.json`.

**Verificações:** SHA-1 do ZIP `21e0559015c56c403daf8933e5040504a7b83837`; SHA-256 da cassette após CRLF→LF `c14b3e2a55295f9a4e88b4e04d4fe503e0ad3eafdf28b6eb9abe15860ec100e5`. Commit de captura: `f3f6c3124d4a9c8793d112701362b60e813cc88e`.

**Observação:** 16 interações HTTP; métodos GET: 12, POST: 4.

## Operações HTTP registradas

- `documento_escolher_tipo`: 4 requisição(ões)
- `procedimento_escolher_tipo`: 3 requisição(ões)
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
