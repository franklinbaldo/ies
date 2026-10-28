---
type: Cassette
title: processo-anexar-ufrj
installation: ufrj
source_zip: https://archive.org/download/todos-sei-cassettes-2026/lotes/lote-c7a4af8d9d70ddc3.zip
archive_member: processo-anexar-ufrj.json
archive_sha1: c7a4af8d9d70ddc3599cdd9bccbb98e6d726db78
cassette_sha256_lf: 0fa4ed4ee794ea3700faf01f6f431cda201e0fe23e059016afdb523758473dab
interactions: 51
schema_version: 1
complete: true
---

# processo-anexar-ufrj

**Instalação:** ufrj (`teste.sei.tic.ufrj.br`). **Origem:** [ZIP imutável do Internet Archive](https://archive.org/download/todos-sei-cassettes-2026/lotes/lote-c7a4af8d9d70ddc3.zip); membro `processo-anexar-ufrj.json`.

**Verificações:** SHA-1 do ZIP `c7a4af8d9d70ddc3599cdd9bccbb98e6d726db78`; SHA-256 da cassette após CRLF→LF `0fa4ed4ee794ea3700faf01f6f431cda201e0fe23e059016afdb523758473dab`. Commit de captura: `3754ebe7fbe1da7622c0550689dcd66cdb1326d6`.

**Observação:** 51 interações HTTP; métodos GET: 31, POST: 20.

## Operações HTTP registradas

- `procedimento_visualizar`: 9 requisição(ões)
- `procedimento_escolher_tipo`: 8 requisição(ões)
- `procedimento_anexar`: 7 requisição(ões)
- `procedimento_gerar`: 6 requisição(ões)
- `protocolo_RI1023`: 4 requisição(ões)
- `procedimento_anexacao_verificar`: 4 requisição(ões)
- `procedimento_trabalhar`: 2 requisição(ões)
- `documento_escolher_tipo`: 2 requisição(ões)
- `documento_gerar`: 2 requisição(ões)
- `procedimento_concluir`: 2 requisição(ões)
- `procedimento_controlar`: 1 requisição(ões)
- `arvore_visualizar`: 1 requisição(ões)
- `editor_montar`: 1 requisição(ões)
- `editor_salvar`: 1 requisição(ões)
- `documento_imprimir_web`: 1 requisição(ões)

## Alcance da evidência

A gravação comprova respostas observadas na instalação indicada. Não demonstra,
por si só, paridade com a CLI IES, corpo de requisições POST ou sucesso sem
conferência pós-escrita. Use os passos relevantes e o contrato antes de implementar.

Consulte [como o SEI funciona](../../docs/como-o-sei-funciona/README.md) e
[o procedimento de replay](../../docs/cassettes.md).
