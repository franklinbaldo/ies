---
type: Cassette
title: processo-enviar-grupo-pen1
installation: pen1
source_zip: https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-pen1.zip
archive_member: processo-enviar-grupo-pen1.json
archive_sha1: 70841f418106b058c4f81c9b13a1322c51ce8cd0
cassette_sha256_lf: aaa9bf747035ccecfb68756f0468b2ef60cefd28d54a519b83c88364fe1f9fc4
interactions: 36
schema_version: 1
complete: true
---

# processo-enviar-grupo-pen1

**Instalação:** pen1 (`sei.orgao1.tramita.processoeletronico.gov.br`). **Origem:** [ZIP imutável do Internet Archive](https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-pen1.zip); membro `processo-enviar-grupo-pen1.json`.

**Verificações:** SHA-1 do ZIP `70841f418106b058c4f81c9b13a1322c51ce8cd0`; SHA-256 da cassette após CRLF→LF `aaa9bf747035ccecfb68756f0468b2ef60cefd28d54a519b83c88364fe1f9fc4`. Commit de captura: `7c1387d764d010cee8a9c6cf44c2a550f3e8701d`.

**Observação:** 36 interações HTTP; métodos GET: 27, POST: 9.

## Operações HTTP registradas

- `grupo_unidade_listar`: 7 requisição(ões)
- `procedimento_visualizar`: 5 requisição(ões)
- `procedimento_enviar`: 5 requisição(ões)
- `unidade_auto_completar_envio_processo`: 4 requisição(ões)
- `grupo_unidade_consultar`: 3 requisição(ões)
- `procedimento_gerar`: 2 requisição(ões)
- `grupo_unidade_cadastrar`: 2 requisição(ões)
- `procedimento_consultar_historico`: 2 requisição(ões)
- `procedimento_controlar`: 1 requisição(ões)
- `procedimento_escolher_tipo`: 1 requisição(ões)
- `procedimento_trabalhar`: 1 requisição(ões)
- `grupo_unidade_excluir`: 1 requisição(ões)
- `arvore_visualizar`: 1 requisição(ões)
- `procedimento_concluir`: 1 requisição(ões)

## Alcance da evidência

A gravação comprova respostas observadas na instalação indicada. Não demonstra,
por si só, paridade com a CLI IES, corpo de requisições POST ou sucesso sem
conferência pós-escrita. Use os passos relevantes e o contrato antes de implementar.

Consulte [como o SEI funciona](../../docs/como-o-sei-funciona/README.md) e
[o procedimento de replay](../../docs/cassettes.md).
