---
type: Cassette
title: unidade-listar-destinos-go
installation: go
source_zip: https://archive.org/download/todos-sei-cassettes-2026/lotes/lote-e01c45d573ff1591.zip
archive_member: unidade-listar-destinos-go.json
archive_sha1: e01c45d573ff15912f0a5d018165aa3edbf24c89
cassette_sha256_lf: 965da599b3e474c2a5b11e9a4f1a30f3ac87a0dc47c7b344213050070404db6b
interactions: 17
schema_version: 1
complete: true
---

# unidade-listar-destinos-go

**Instalação:** go (`seitreinamento.go.gov.br`). **Origem:** [ZIP imutável do Internet Archive](https://archive.org/download/todos-sei-cassettes-2026/lotes/lote-e01c45d573ff1591.zip); membro `unidade-listar-destinos-go.json`.

**Verificações:** SHA-1 do ZIP `e01c45d573ff15912f0a5d018165aa3edbf24c89`; SHA-256 da cassette após CRLF→LF `965da599b3e474c2a5b11e9a4f1a30f3ac87a0dc47c7b344213050070404db6b`. Commit de captura: `3db5e18793622fdff6be793ad86d85617811ee10`.

**Observação:** 17 interações HTTP; métodos GET: 13, POST: 4.

## Operações HTTP registradas

- `unidade_selecionar_envio_processo`: 6 requisição(ões)
- `grupo_unidade_listar`: 3 requisição(ões)
- `grupo_unidade_cadastrar`: 3 requisição(ões)
- `unidade_selecionar_outras`: 2 requisição(ões)
- `procedimento_controlar`: 1 requisição(ões)
- `bloco_assinatura_listar`: 1 requisição(ões)
- `bloco_assinatura_cadastrar`: 1 requisição(ões)

## Alcance da evidência

A gravação comprova respostas observadas na instalação indicada. Não demonstra,
por si só, paridade com a CLI IES, corpo de requisições POST ou sucesso sem
conferência pós-escrita. Use os passos relevantes e o contrato antes de implementar.

Consulte [como o SEI funciona](../../docs/como-o-sei-funciona/README.md) e
[o procedimento de replay](../../docs/cassettes.md).
