---
type: Cassette
title: unidade-listar-destinos-ufrj
installation: ufrj
source_zip: https://archive.org/download/todos-sei-cassettes-2026/lotes/lote-e01c45d573ff1591.zip
archive_member: unidade-listar-destinos-ufrj.json
archive_sha1: e01c45d573ff15912f0a5d018165aa3edbf24c89
cassette_sha256_lf: f71c022a210154280df1bba92d554f200c9412c24f608a3041076f70b231a302
interactions: 17
schema_version: 1
complete: true
---

# unidade-listar-destinos-ufrj

**Instalação:** ufrj (`teste.sei.tic.ufrj.br`). **Origem:** [ZIP imutável do Internet Archive](https://archive.org/download/todos-sei-cassettes-2026/lotes/lote-e01c45d573ff1591.zip); membro `unidade-listar-destinos-ufrj.json`.

**Verificações:** SHA-1 do ZIP `e01c45d573ff15912f0a5d018165aa3edbf24c89`; SHA-256 da cassette após CRLF→LF `f71c022a210154280df1bba92d554f200c9412c24f608a3041076f70b231a302`. Commit de captura: `9dd7ecabe8591eb4aaab59b90aeb4b5f68892cc6`.

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
