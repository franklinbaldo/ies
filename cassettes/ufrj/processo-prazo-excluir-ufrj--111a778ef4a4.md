---
type: Cassette
title: processo-prazo-excluir-ufrj
installation: ufrj
source_zip: https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-ufrj.zip
archive_member: processo-prazo-excluir-ufrj.json
archive_sha1: 3b29ba3c4e18fa0b336fbec9490c4f59a15d4aa0
cassette_sha256_lf: 111a778ef4a46263a97c75b246d156bcdafa211e6906a01be58c0f61fda598b6
interactions: 25
schema_version: 1
complete: true
---

# processo-prazo-excluir-ufrj

**Instalação:** ufrj (`teste.sei.tic.ufrj.br`). **Origem:** [ZIP imutável do Internet Archive](https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-ufrj.zip); membro `processo-prazo-excluir-ufrj.json`.

**Verificações:** SHA-1 do ZIP `3b29ba3c4e18fa0b336fbec9490c4f59a15d4aa0`; SHA-256 da cassette após CRLF→LF `111a778ef4a46263a97c75b246d156bcdafa211e6906a01be58c0f61fda598b6`. Commit de captura: `461886b422f372314dfb28df3b28010113e29a16`.

**Observação:** 25 interações HTTP; métodos GET: 19, POST: 6.

## Operações HTTP registradas

- `procedimento_visualizar`: 5 requisição(ões)
- `controle_prazo_listar`: 5 requisição(ões)
- `procedimento_escolher_tipo`: 4 requisição(ões)
- `procedimento_gerar`: 3 requisição(ões)
- `controle_prazo_definir`: 3 requisição(ões)
- `procedimento_concluir`: 2 requisição(ões)
- `procedimento_controlar`: 1 requisição(ões)
- `procedimento_trabalhar`: 1 requisição(ões)
- `controle_prazo_excluir`: 1 requisição(ões)

## Alcance da evidência

A gravação comprova respostas observadas na instalação indicada. Não demonstra,
por si só, paridade com a CLI IES, corpo de requisições POST ou sucesso sem
conferência pós-escrita. Use os passos relevantes e o contrato antes de implementar.

Consulte [como o SEI funciona](../../docs/como-o-sei-funciona/README.md) e
[o procedimento de replay](../../docs/cassettes.md).
