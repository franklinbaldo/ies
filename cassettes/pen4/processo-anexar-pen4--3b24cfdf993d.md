---
type: Cassette
title: processo-anexar-pen4
installation: pen4
source_zip: https://archive.org/download/todos-sei-cassettes-2026/lotes/lote-0306b45276ae0084.zip
archive_member: processo-anexar-pen4.json
archive_sha1: 0306b45276ae0084b78e781051f7ab5fc07e2693
cassette_sha256_lf: 3b24cfdf993d2d3940c8441286726f276563873d7070f9fb3db2962ad623b9f8
interactions: 52
schema_version: 1
complete: true
---

# processo-anexar-pen4

**Instalação:** pen4 (`sei.orgao4.tramita.processoeletronico.gov.br`). **Origem:** [ZIP imutável do Internet Archive](https://archive.org/download/todos-sei-cassettes-2026/lotes/lote-0306b45276ae0084.zip); membro `processo-anexar-pen4.json`.

**Verificações:** SHA-1 do ZIP `0306b45276ae0084b78e781051f7ab5fc07e2693`; SHA-256 da cassette após CRLF→LF `3b24cfdf993d2d3940c8441286726f276563873d7070f9fb3db2962ad623b9f8`. Commit de captura: `f678e9846926faa4b34caec8388be873ffa466e1`.

**Observação:** 52 interações HTTP; métodos GET: 32, POST: 20.

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
- `editor_salvar_conteudo`: 1 requisição(ões)
- `documento_imprimir_web`: 1 requisição(ões)
- `documento_visualizar`: 1 requisição(ões)

## Alcance da evidência

A gravação comprova respostas observadas na instalação indicada. Não demonstra,
por si só, paridade com a CLI IES, corpo de requisições POST ou sucesso sem
conferência pós-escrita. Use os passos relevantes e o contrato antes de implementar.

Consulte [como o SEI funciona](../../docs/como-o-sei-funciona/README.md) e
[o procedimento de replay](../../docs/cassettes.md).
