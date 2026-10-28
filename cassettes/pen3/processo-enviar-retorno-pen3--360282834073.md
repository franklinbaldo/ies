---
type: Cassette
title: processo-enviar-retorno-pen3
installation: pen3
source_zip: https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-pen3.zip
archive_member: processo-enviar-retorno-pen3.json
archive_sha1: b81b84fca29299f2096fb128593ba52f2ac9a7ab
cassette_sha256_lf: 360282834073fd44ce35d8be49fd474e42fe6424986ab637a7e6d807cb92e71b
interactions: 24
schema_version: 1
complete: true
---

# processo-enviar-retorno-pen3

**Instalação:** pen3 (`sei.orgao3.tramita.processoeletronico.gov.br`). **Origem:** [ZIP imutável do Internet Archive](https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-pen3.zip); membro `processo-enviar-retorno-pen3.json`.

**Verificações:** SHA-1 do ZIP `b81b84fca29299f2096fb128593ba52f2ac9a7ab`; SHA-256 da cassette após CRLF→LF `360282834073fd44ce35d8be49fd474e42fe6424986ab637a7e6d807cb92e71b`. Commit de captura: `440fcdd824c6099021daf9671629cc258a71584d`.

**Observação:** 24 interações HTTP; métodos GET: 17, POST: 7.

## Operações HTTP registradas

- `procedimento_escolher_tipo`: 4 requisição(ões)
- `procedimento_visualizar`: 4 requisição(ões)
- `procedimento_gerar`: 3 requisição(ões)
- `procedimento_enviar`: 3 requisição(ões)
- `retorno_programado_listar`: 3 requisição(ões)
- `unidade_auto_completar_envio_processo`: 2 requisição(ões)
- `procedimento_concluir`: 2 requisição(ões)
- `procedimento_controlar`: 1 requisição(ões)
- `procedimento_trabalhar`: 1 requisição(ões)
- `procedimento_consultar_historico`: 1 requisição(ões)

## Alcance da evidência

A gravação comprova respostas observadas na instalação indicada. Não demonstra,
por si só, paridade com a CLI IES, corpo de requisições POST ou sucesso sem
conferência pós-escrita. Use os passos relevantes e o contrato antes de implementar.

Consulte [como o SEI funciona](../../docs/como-o-sei-funciona/README.md) e
[o procedimento de replay](../../docs/cassettes.md).
