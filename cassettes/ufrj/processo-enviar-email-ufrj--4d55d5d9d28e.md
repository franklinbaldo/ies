---
type: Cassette
title: processo-enviar-email-ufrj
installation: ufrj
source_zip: https://archive.org/download/todos-sei-cassettes-2026/lotes/lote-22044799039c3b65.zip
archive_member: processo-enviar-email-ufrj.json
archive_sha1: 22044799039c3b65aba900b43f4cc826fd2725c2
cassette_sha256_lf: 4d55d5d9d28e34369dd23a17212e53e407fe860801f9bf398a7ef5d51b49ccdb
interactions: 22
schema_version: 1
complete: true
---

# processo-enviar-email-ufrj

**Instalação:** ufrj (`teste.sei.tic.ufrj.br`). **Origem:** [ZIP imutável do Internet Archive](https://archive.org/download/todos-sei-cassettes-2026/lotes/lote-22044799039c3b65.zip); membro `processo-enviar-email-ufrj.json`.

**Verificações:** SHA-1 do ZIP `22044799039c3b65aba900b43f4cc826fd2725c2`; SHA-256 da cassette após CRLF→LF `4d55d5d9d28e34369dd23a17212e53e407fe860801f9bf398a7ef5d51b49ccdb`. Commit de captura: `645620abe4a215653a2556903defa104957e5845`.

**Observação:** 22 interações HTTP; métodos GET: 18, POST: 4.

## Operações HTTP registradas

- `procedimento_visualizar`: 7 requisição(ões)
- `procedimento_enviar_email`: 3 requisição(ões)
- `procedimento_escolher_tipo`: 2 requisição(ões)
- `procedimento_gerar`: 2 requisição(ões)
- `arvore_visualizar`: 2 requisição(ões)
- `procedimento_consultar_historico`: 2 requisição(ões)
- `procedimento_concluir`: 2 requisição(ões)
- `procedimento_controlar`: 1 requisição(ões)
- `procedimento_trabalhar`: 1 requisição(ões)

## Alcance da evidência

A gravação comprova respostas observadas na instalação indicada. Não demonstra,
por si só, paridade com a CLI IES, corpo de requisições POST ou sucesso sem
conferência pós-escrita. Use os passos relevantes e o contrato antes de implementar.

Consulte [como o SEI funciona](../../docs/como-o-sei-funciona/README.md) e
[o procedimento de replay](../../docs/cassettes.md).
