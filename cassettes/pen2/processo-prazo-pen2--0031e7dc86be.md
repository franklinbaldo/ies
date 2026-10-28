---
type: Cassette
title: processo-prazo-pen2
installation: pen2
source_zip: https://archive.org/download/todos-sei-cassettes-2026/lotes/lote-0306b45276ae0084.zip
archive_member: processo-prazo-pen2.json
archive_sha1: 0306b45276ae0084b78e781051f7ab5fc07e2693
cassette_sha256_lf: 0031e7dc86bea7f4d93555777ef6f40a3dc33247ea7890453a182cd322e3e11a
interactions: 35
schema_version: 1
complete: true
---

# processo-prazo-pen2

**Instalação:** pen2 (`sei.orgao2.tramita.processoeletronico.gov.br`). **Origem:** [ZIP imutável do Internet Archive](https://archive.org/download/todos-sei-cassettes-2026/lotes/lote-0306b45276ae0084.zip); membro `processo-prazo-pen2.json`.

**Verificações:** SHA-1 do ZIP `0306b45276ae0084b78e781051f7ab5fc07e2693`; SHA-256 da cassette após CRLF→LF `0031e7dc86bea7f4d93555777ef6f40a3dc33247ea7890453a182cd322e3e11a`. Commit de captura: `4bfd3ec87842d40878374bc8bded3e2f6a2fa9da`.

**Observação:** 35 interações HTTP; métodos GET: 28, POST: 7.

## Operações HTTP registradas

- `procedimento_visualizar`: 12 requisição(ões)
- `controle_prazo_definir`: 12 requisição(ões)
- `procedimento_escolher_tipo`: 4 requisição(ões)
- `procedimento_gerar`: 3 requisição(ões)
- `procedimento_concluir`: 2 requisição(ões)
- `procedimento_controlar`: 1 requisição(ões)
- `procedimento_trabalhar`: 1 requisição(ões)

## Alcance da evidência

A gravação comprova respostas observadas na instalação indicada. Não demonstra,
por si só, paridade com a CLI IES, corpo de requisições POST ou sucesso sem
conferência pós-escrita. Use os passos relevantes e o contrato antes de implementar.

Consulte [como o SEI funciona](../../docs/como-o-sei-funciona/README.md) e
[o procedimento de replay](../../docs/cassettes.md).
