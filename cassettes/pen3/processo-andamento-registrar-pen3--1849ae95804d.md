---
type: Cassette
title: processo-andamento-registrar-pen3
installation: pen3
source_zip: https://archive.org/download/todos-sei-cassettes-2026/lotes/lote-0306b45276ae0084.zip
archive_member: processo-andamento-registrar-pen3.json
archive_sha1: 0306b45276ae0084b78e781051f7ab5fc07e2693
cassette_sha256_lf: 1849ae95804d54b4f0cc2e453572794b09560bc2b2be19a8875b040eb3feb781
interactions: 25
schema_version: 1
complete: true
---

# processo-andamento-registrar-pen3

**Instalação:** pen3 (`sei.orgao3.tramita.processoeletronico.gov.br`). **Origem:** [ZIP imutável do Internet Archive](https://archive.org/download/todos-sei-cassettes-2026/lotes/lote-0306b45276ae0084.zip); membro `processo-andamento-registrar-pen3.json`.

**Verificações:** SHA-1 do ZIP `0306b45276ae0084b78e781051f7ab5fc07e2693`; SHA-256 da cassette após CRLF→LF `1849ae95804d54b4f0cc2e453572794b09560bc2b2be19a8875b040eb3feb781`. Commit de captura: `7a2479a68a6820cc4245111b87d3716a0348fd8a`.

**Observação:** 25 interações HTTP; métodos GET: 19, POST: 6.

## Operações HTTP registradas

- `procedimento_visualizar`: 6 requisição(ões)
- `procedimento_atualizar_andamento`: 5 requisição(ões)
- `procedimento_escolher_tipo`: 4 requisição(ões)
- `procedimento_gerar`: 3 requisição(ões)
- `procedimento_consultar_historico`: 3 requisição(ões)
- `procedimento_concluir`: 2 requisição(ões)
- `procedimento_controlar`: 1 requisição(ões)
- `procedimento_trabalhar`: 1 requisição(ões)

## Alcance da evidência

A gravação comprova respostas observadas na instalação indicada. Não demonstra,
por si só, paridade com a CLI IES, corpo de requisições POST ou sucesso sem
conferência pós-escrita. Use os passos relevantes e o contrato antes de implementar.

Consulte [como o SEI funciona](../../docs/como-o-sei-funciona/README.md) e
[o procedimento de replay](../../docs/cassettes.md).
