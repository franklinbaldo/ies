---
type: Cassette
title: cadastro-busca-unb
installation: unb
source_zip: https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-unb.zip
archive_member: cadastro-busca-unb.json
archive_sha1: 21e0559015c56c403daf8933e5040504a7b83837
cassette_sha256_lf: f93ca43f297857b8f794ece4a1362db9420c9762349d2fbb8fc721135dd3ad86
interactions: 7
schema_version: 1
complete: true
---

# cadastro-busca-unb

**Instalação:** unb (`sei.treinamento.unb.br`). **Origem:** [ZIP imutável do Internet Archive](https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-unb.zip); membro `cadastro-busca-unb.json`.

**Verificações:** SHA-1 do ZIP `21e0559015c56c403daf8933e5040504a7b83837`; SHA-256 da cassette após CRLF→LF `f93ca43f297857b8f794ece4a1362db9420c9762349d2fbb8fc721135dd3ad86`. Commit de captura: `bea4a9d3aca6a41477c5c9cb683c5548d4d41aed`.

**Observação:** 7 interações HTTP; métodos GET: 4, POST: 3.

## Operações HTTP registradas

- `protocolo_pesquisar`: 3 requisição(ões)
- `contato_auto_completar_pesquisa`: 2 requisição(ões)
- `procedimento_controlar`: 1 requisição(ões)
- `assunto_auto_completar_RI1223`: 1 requisição(ões)

## Alcance da evidência

A gravação comprova respostas observadas na instalação indicada. Não demonstra,
por si só, paridade com a CLI IES, corpo de requisições POST ou sucesso sem
conferência pós-escrita. Use os passos relevantes e o contrato antes de implementar.

Consulte [como o SEI funciona](../../docs/como-o-sei-funciona/README.md) e
[o procedimento de replay](../../docs/cassettes.md).
