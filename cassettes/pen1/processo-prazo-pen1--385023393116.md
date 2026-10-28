---
type: Cassette
title: processo-prazo-pen1
installation: pen1
source_zip: https://archive.org/download/todos-sei-cassettes-2026/lotes/lote-0306b45276ae0084.zip
archive_member: processo-prazo-pen1.json
archive_sha1: 0306b45276ae0084b78e781051f7ab5fc07e2693
cassette_sha256_lf: 3850233931164181636b332ed7bd8467880b33190e421580e8a3088dc33eccf6
interactions: 33
schema_version: 1
complete: true
---

# processo-prazo-pen1

**Instalação:** pen1 (`sei.orgao1.tramita.processoeletronico.gov.br`). **Origem:** [ZIP imutável do Internet Archive](https://archive.org/download/todos-sei-cassettes-2026/lotes/lote-0306b45276ae0084.zip); membro `processo-prazo-pen1.json`.

**Verificações:** SHA-1 do ZIP `0306b45276ae0084b78e781051f7ab5fc07e2693`; SHA-256 da cassette após CRLF→LF `3850233931164181636b332ed7bd8467880b33190e421580e8a3088dc33eccf6`. Commit de captura: `3fa42f0fb72f84c522b89dffed987a76c2f3a4fa`.

**Observação:** 33 interações HTTP; métodos GET: 29, POST: 4.

## Operações HTTP registradas

- `procedimento_visualizar`: 12 requisição(ões)
- `controle_prazo_definir`: 12 requisição(ões)
- `procedimento_gerar`: 3 requisição(ões)
- `procedimento_escolher_tipo`: 2 requisição(ões)
- `procedimento_controlar`: 1 requisição(ões)
- `procedimento_trabalhar`: 1 requisição(ões)
- `arvore_visualizar`: 1 requisição(ões)
- `procedimento_concluir`: 1 requisição(ões)

## Alcance da evidência

A gravação comprova respostas observadas na instalação indicada. Não demonstra,
por si só, paridade com a CLI IES, corpo de requisições POST ou sucesso sem
conferência pós-escrita. Use os passos relevantes e o contrato antes de implementar.

Consulte [como o SEI funciona](../../docs/como-o-sei-funciona/README.md) e
[o procedimento de replay](../../docs/cassettes.md).
