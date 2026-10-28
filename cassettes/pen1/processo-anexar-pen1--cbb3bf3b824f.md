---
type: Cassette
title: processo-anexar-pen1
installation: pen1
source_zip: https://archive.org/download/todos-sei-cassettes-2026/lotes/lote-0306b45276ae0084.zip
archive_member: processo-anexar-pen1.json
archive_sha1: 0306b45276ae0084b78e781051f7ab5fc07e2693
cassette_sha256_lf: cbb3bf3b824f0e201aa0d0da03930a3b47d39da26e49d169c7f814683333422a
interactions: 47
schema_version: 1
complete: true
---

# processo-anexar-pen1

**Instalação:** pen1 (`sei.orgao1.tramita.processoeletronico.gov.br`). **Origem:** [ZIP imutável do Internet Archive](https://archive.org/download/todos-sei-cassettes-2026/lotes/lote-0306b45276ae0084.zip); membro `processo-anexar-pen1.json`.

**Verificações:** SHA-1 do ZIP `0306b45276ae0084b78e781051f7ab5fc07e2693`; SHA-256 da cassette após CRLF→LF `cbb3bf3b824f0e201aa0d0da03930a3b47d39da26e49d169c7f814683333422a`. Commit de captura: `a7dd37a08db1af982ddbb169f589f67d1d0aadc1`.

**Observação:** 47 interações HTTP; métodos GET: 33, POST: 14.

## Operações HTTP registradas

- `procedimento_visualizar`: 9 requisição(ões)
- `procedimento_anexar`: 7 requisição(ões)
- `procedimento_gerar`: 6 requisição(ões)
- `procedimento_escolher_tipo`: 4 requisição(ões)
- `protocolo_RI1023`: 4 requisição(ões)
- `procedimento_anexacao_verificar`: 4 requisição(ões)
- `procedimento_trabalhar`: 2 requisição(ões)
- `documento_gerar`: 2 requisição(ões)
- `arvore_visualizar`: 2 requisição(ões)
- `procedimento_controlar`: 1 requisição(ões)
- `documento_escolher_tipo`: 1 requisição(ões)
- `editor_montar`: 1 requisição(ões)
- `editor_salvar`: 1 requisição(ões)
- `documento_imprimir_web`: 1 requisição(ões)
- `documento_visualizar`: 1 requisição(ões)
- `procedimento_concluir`: 1 requisição(ões)

## Alcance da evidência

A gravação comprova respostas observadas na instalação indicada. Não demonstra,
por si só, paridade com a CLI IES, corpo de requisições POST ou sucesso sem
conferência pós-escrita. Use os passos relevantes e o contrato antes de implementar.

Consulte [como o SEI funciona](../../docs/como-o-sei-funciona/README.md) e
[o procedimento de replay](../../docs/cassettes.md).
